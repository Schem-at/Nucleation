# Releases

Keep `Cargo.toml`, `Cargo.lock`, and `bindings/python/pyproject.toml` on the same
release version, update `RELEASE_NOTES.md`, and push the matching `vX.Y.Z` tag.
The `Nucleation CI/CD` workflow builds and tests all supported bindings before
publishing. npm and JVM versions are stamped from the Cargo version by CI.

## Recover a partially published release

Keep the original release tag unchanged. In the **Nucleation CI/CD** workflow,
choose **Run workflow** on `master` and set `resume_run_id` to the original tag
run's numeric ID. Alternatively, push the current workflow commit to a branch
named `release-recovery/<run-id>`:

```sh
git push origin HEAD:refs/heads/release-recovery/43438134223
```

That example resumes the original v0.12.2 run; use the actual source run ID for
other versions. Recovery validates that the source was a completed release-tag
run in this repository, the tag still points to its source commit, and every
required test and platform build succeeded. It downloads that run's retained
artifacts and checks out its source commit for Rust publication. Expired or
missing artifacts must be rebuilt through a new validated release run.

Existing Rust, npm, and Python publications are skipped. A failed npm or PyPI
upload does not prevent the remaining package and GitHub release uploads; the
workflow still ends in failure when any registry upload fails. Diagnose that
registry's error before retrying. Recovery uses `ci.yml` so npm's trusted
publisher workflow identity is preserved.

After publication, verify the package versions on npm, PyPI, and crates.io,
and check the native archives and JVM JAR on the GitHub release. For simulator
releases, exercise the published Python and JS packages against the vanilla
regression fixture as well as the source tests.
