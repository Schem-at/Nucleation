# Notice — lithium-derived test structures

The `.litematic` files in this directory embed block data derived from the
gametest structures of [CaffeineMC/lithium](https://github.com/CaffeineMC/lithium)
(`common/src/gametest/resources/data/lithium-gametest/`), licensed under the
**GNU LGPL-3.0**, at pinned commit
`c42972b6e9d21c8ff45559df6b271802050a22e2` (develop, fetched 2026-08-03).

They are **not** covered by this repository's MIT licence: to the extent the
embedded structures are copyrightable, they remain under the LGPL-3.0
(see https://www.gnu.org/licenses/lgpl-3.0.html).

Each file was produced by `nucleation port`: the structure is converted from
vanilla structure-SNBT to the Litematica format, and a test descriptor (this
repository's own authorship, from `tests/corpus/lithium-specs/` or
synthesized from the structure's `test_block`s) is embedded in the root
`NucleationTest` tag. Regenerate at any time with:

```sh
tools/fetch-lithium-gametests.sh
cargo run -p nucleation-cli -- \
    port --path tests/corpus/lithium --specs tests/corpus/lithium-specs \
         --out tests/corpus/lithium-litematic
```
