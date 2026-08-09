#!/usr/bin/env bash
# Build-environment health check. Answers "why is my loop slow?" in one screen.
#
#   tools/doctor.sh          report + warn
#   tools/doctor.sh --quiet  only print problems (for hooks / CI)
#
# Thresholds are set where this repo actually started hurting: a 102 GB target
# with 774k files in debug/deps made a NO-OP `cargo check` take longer than 30s
# purely scanning fingerprints.
set -uo pipefail
cd "$(dirname "$0")/.."

QUIET=0
[[ "${1:-}" == "--quiet" ]] && QUIET=1

WARNINGS=0
ok()   { ((QUIET)) || printf '  \033[32mok\033[0m    %s\n' "$1"; }
warn() { printf '  \033[33mwarn\033[0m  %s\n' "$1"; WARNINGS=$((WARNINGS + 1)); }
info() { ((QUIET)) || printf '        %s\n' "$1"; }
head_() { ((QUIET)) || printf '\n\033[1m%s\033[0m\n' "$1"; }

# ------------------------------------------------------------------- disk
head_ "Disk"
DISK_AVAIL_G=$(df -g . | awk 'NR==2{print $4}')
DISK_PCT=$(df . | awk 'NR==2{gsub("%","",$5); print $5}')
if ((DISK_AVAIL_G < 25)); then
  warn "only ${DISK_AVAIL_G} GiB free (${DISK_PCT}% used) — prune: tools/dev.sh doctor then cargo clean"
elif ((DISK_PCT > 90)); then
  warn "disk ${DISK_PCT}% full (${DISK_AVAIL_G} GiB free)"
else
  ok "${DISK_AVAIL_G} GiB free (${DISK_PCT}% used)"
fi

# ----------------------------------------------------------------- target
head_ "Target directory"
if [[ -d target ]]; then
  TARGET_K=$(du -sk target 2>/dev/null | cut -f1)
  TARGET_G=$((TARGET_K / 1048576))
  if ((TARGET_G > 40)); then
    warn "target/ is ${TARGET_G} GB — prune with: cargo clean (or: rm -rf target/debug/incremental)"
  elif ((TARGET_G > 20)); then
    info "target/ is ${TARGET_G} GB (watch it)"
    ok "target/ size acceptable"
  else
    ok "target/ is ${TARGET_G} GB"
  fi

  if [[ -d target/debug/deps ]]; then
    DEPS_N=$(/bin/ls target/debug/deps 2>/dev/null | wc -l | tr -d ' ')
    if ((DEPS_N > 200000)); then
      warn "${DEPS_N} files in target/debug/deps — fingerprint scanning alone will cost seconds per no-op check; cargo clean"
    else
      ok "${DEPS_N} files in target/debug/deps"
    fi
  fi

  if [[ -d target/debug/incremental ]]; then
    INC_K=$(du -sk target/debug/incremental 2>/dev/null | cut -f1)
    INC_G=$((INC_K / 1048576))
    ((INC_G > 5)) \
      && warn "incremental cache is ${INC_G} GB — safe to delete: rm -rf target/debug/incremental" \
      || ok "incremental cache ${INC_G} GB"
  fi
else
  ok "no target/ yet"
fi

# ---------------------------------------------------------------- sccache
head_ "Compiler cache"
if command -v sccache >/dev/null 2>&1; then
  # `cargo config get` is nightly-only, so read the committed config directly.
  if grep -qE '^\s*rustc-wrapper\s*=\s*"[^"]*sccache"' .cargo/config.toml 2>/dev/null \
     || [[ -n "${RUSTC_WRAPPER:-}" ]]; then
    ok "sccache active ($(sccache --version))"
    if ((!QUIET)); then
      sccache -s 2>/dev/null | grep -Ei 'compile requests|cache hits|cache misses|cache size' \
        | sed 's/^/        /'
    fi
  else
    warn "sccache installed but NOT wired up — expected build.rustc-wrapper in .cargo/config.toml"
  fi
else
  warn "sccache missing — install: brew install sccache (caches the 159 dependency builds across feature sets)"
fi

# ----------------------------------------------------------------- linker
head_ "Linker"
# On Apple Silicon the Xcode default is ld-prime (ld64 >= 1000), which is
# already the fast linker — lld is NOT a win here and is not installed.
LD_V=$(ld -v 2>&1 | head -1)
if [[ "$LD_V" =~ PROJECT:ld-([0-9]+) ]]; then
  LD_MAJOR="${BASH_REMATCH[1]}"
  ((LD_MAJOR >= 1000)) \
    && ok "Apple ld-prime (ld-${LD_MAJOR}) — already the fast path on arm64; lld not needed" \
    || warn "old Apple ld (${LD_MAJOR}); consider installing lld"
else
  info "linker: $LD_V"
fi

# --------------------------------------------------------------- debuginfo
head_ "Dev profile"
if grep -qE '^\s*debug\s*=\s*(0|false|"line-tables-only")' Cargo.toml 2>/dev/null; then
  ok "dev profile trims debuginfo (this is what keeps target/ small)"
else
  warn "[profile.dev] still emits full debuginfo — deps at debug=2 with split-debuginfo=unpacked is what grows target/ to 100 GB"
fi

# ------------------------------------------------- concurrent cargo / IDE
head_ "Concurrency"
# Two cargo invocations with DIFFERENT feature sets block each other on the
# build-directory lock and each maintain their own artifacts. RustRover's
# flycheck (`cargo check --workspace --all-targets`) is the usual offender.
CARGOS=$(pgrep -fl 'bin/cargo (check|build|test)' 2>/dev/null | wc -l | tr -d ' ')
if ((CARGOS > 1)); then
  warn "${CARGOS} cargo processes running — they serialise on the build-dir lock"
  ((QUIET)) || pgrep -fl 'bin/cargo (check|build|test)' 2>/dev/null \
    | sed -E 's/^([0-9]+) .*(check|build|test)/  \1 ... \2/' | cut -c1-140 | sed 's/^/        /'
  info "point the IDE at the canonical feature set (docs/DEV.md) or give it its own --target-dir"
elif ((CARGOS == 1)); then
  info "1 cargo process running"
  ok "no lock contention"
else
  ok "no cargo processes running"
fi

# ------------------------------------------------------------------ result
if ((WARNINGS)); then
  printf '\n\033[33m%d warning(s)\033[0m — see docs/DEV.md "Disk hygiene"\n' "$WARNINGS"
  exit 1
fi
((QUIET)) || printf '\n\033[32mbuild environment healthy\033[0m\n'
