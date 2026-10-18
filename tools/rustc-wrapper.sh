#!/bin/sh
# build.rustc-wrapper (see .cargo/config.toml). Cargo runs this as
# `rustc-wrapper.sh <rustc> <args...>`: route through sccache when it is
# installed, otherwise run rustc directly, so a fresh machine builds without it.
if command -v sccache >/dev/null 2>&1; then
  exec sccache "$@"
fi
exec "$@"
