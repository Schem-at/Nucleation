# Fork-surface reduction: what it takes

The binding pipeline depends on two fragile pins that can break a future
release for reasons unrelated to our code. This doc scopes the exit paths.
(Status as of 2026-07-18.)

## 1. The diplomat fork (PHP backend)

**Dependency:** `gen-bindings.sh` requires `diplomat-tool` from
`Nano112/diplomat` branch `php-backend`. Upstream diplomat-tool has no PHP
target; installing upstream silently shadows the fork and `gen-bindings.sh`
dies *after* wiping `bindings/php` (it did exactly this on 2026-07-17).

**Exit path — upstream the PHP backend:**
- Upstream (rust-diplomat/diplomat) accepts new backends as
  `tool/src/<lang>/` modules; the demo_gen and nanobind backends both landed
  as PRs from outside the core team, so there is precedent.
- Work: rebase `php-backend` onto upstream main (the fork tracks 0.15.x;
  upstream may have moved), add PHP to their CI matrix + feature_tests
  goldens, and drive the review. The backend itself is self-contained
  (~one directory), which helps.
- Estimate: a few days of rebase/test work plus review round-trips measured
  in weeks. Until merged, keep the fork but pin it by **rev** (not branch)
  in gen-bindings.sh instructions and CI so upstream force-pushes can't
  break regeneration.
- Cheap interim hardening (recommended regardless): make `gen-bindings.sh`
  verify `diplomat-tool` supports the `php` target *before* the `rm -rf`,
  e.g. `diplomat-tool php --help` probe, and fail with the install hint.

## 2. The nanobind ==2.12.0 pin

**Dependency:** the Python wheel pins `nanobind ==2.12.0`
(bindings/python/pyproject.toml and examples/bridge_smoke/python/run.sh)
because diplomat's generated dealloc shim reaches into nanobind's private
`nb_inst` struct, whose layout changed in nanobind 2.13.

**Why it's a time bomb:** every new CPython release eventually needs a
newer nanobind (3.14 support arrived in 2.9.x; a hypothetical 3.15 may
land only in >=2.13). When that day comes the pin becomes unsatisfiable.

**Exit path — get the shim onto public API:**
- The clean fix lives in the diplomat fork's nanobind backend codegen, not
  in nucleation: emit a deleter that goes through supported nanobind
  machinery (`nb::inst_mark_ready`/`nb::inst_state` are public;
  tp_dealloc + `nb::detail::nb_inst` layout is not).
- Concretely: audit what the generated shim reads from `nb_inst` (offset
  bookkeeping to find the C++ payload) and replace with
  `nb::inst_ptr<T>(self)` — public, stable, exactly for this. Then bump the
  pin to `>=2.12,<3` and add a CI leg that builds the wheel against latest
  nanobind to catch drift early.
- Estimate: 1–2 days in the fork's nanobind backend + regenerate + the
  existing wheel smoke tests validate it. This is the higher-value, lower-
  effort item of the two — do it first, and it also shrinks what needs
  upstreaming in (1).

## Order of attack

1. Harden gen-bindings.sh (probe before wipe) — minutes, do immediately.
2. Fix the dealloc shim in the fork's nanobind backend, relax the pin.
3. Rebase + upstream the PHP backend; switch gen-bindings.sh to upstream
   once merged and delete the fork.
