# Nucleation fuzz targets

Install `cargo-fuzz`, then run either bounded entry point from the repository
root:

```sh
cargo fuzz run bounded_decode
cargo fuzz run policy_json
```

`bounded_decode` uses deliberately small allocation ceilings. Crashes,
panics, hangs, and allocations beyond those ceilings are bugs. Seed corpora can
be copied from `tests/samples/` without changing the harness.
