# Nucleation v0.2.9

CI runner fix. v0.2.8 stalled indefinitely on the `build-jvm-cdylib
(macos-x64)` matrix entry because the workflow requested `macos-13`,
which GitHub deprecated mid-2025. The existing `build-nucleation` and
`build-python-wheels` jobs in the same file already use the supported
`macos-15-intel` label — the JVM matrix is now aligned.

No source / API changes since v0.2.7. v0.2.8 was retired without
publishing any artifacts.

See the v0.2.7 release notes for the substantive change list.
