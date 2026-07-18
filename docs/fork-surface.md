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

## 2. The nanobind ==2.12.0 pin — RESOLVED (2026-07-18)

**Was:** the Python wheel pinned `nanobind ==2.12.0`
(bindings/python/pyproject.toml and examples/bridge_smoke/python/run.sh)
because diplomat's generated dealloc shim reached into nanobind's private
`nb_inst` struct, whose layout changed in nanobind 2.13 (the
destruct/cpp_delete bools were repacked into a `nb_inst_state` bitfield).

**Fix (landed):** the fork's nanobind backend now emits the dealloc shim
in terms of nanobind's public low-level instance API only —
`nb::inst_state`, `nb::inst_destruct`, `nb::inst_set_state`,
`nb::inst_ptr<T>` — and no longer includes `<../src/nb_internals.h>`.
`inst_set_state(h, false, false)` derives the internal "free the payload"
flag from `destruct` in both 2.12 and 2.13+, so the chained original
`inst_dealloc` can be prevented from double-freeing without touching the
private struct. Lives on fork branch `nanobind-public-api` (based on
`php-backend`); CI installs diplomat-tool from that branch.

The pin is now `nanobind >=2.12,<3`. Verified by building the wheel and
running a 10k create/drop destruction stress against both nanobind 2.12.0
and 2.13.0 (the old shim did not even compile on 2.13).

Still open (nice-to-have): a CI leg that builds the wheel against latest
nanobind on every run, to catch future drift early.

## Order of attack

1. ~~Harden gen-bindings.sh (probe before wipe)~~ — done.
2. ~~Fix the dealloc shim in the fork's nanobind backend, relax the pin~~ —
   done (branch `nanobind-public-api`, see section 2).
3. Rebase + upstream the PHP backend (including the nanobind shim fix);
   switch gen-bindings.sh to upstream once merged and delete the fork.
