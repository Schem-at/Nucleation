# Versions and translation

## Versions and translation
The built-in DataConverter port migrates blocks, items, and entities across
Minecraft data versions (loss reports on downgrades), and Java ↔ Bedrock
translation runs on GeyserMC's mappings at full **26.2** parity.

Those loss reports are per block, so you can see a downgrade before you commit
it. Here a sampler of blocks from many eras is checked against 1.12.2 (before
the Flattening) and recolored by the verdict: green survives, red is a loss.

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/migration.png" width="820" alt="A sampler of blocks from many Minecraft versions, and the same layout recolored green for survives and red for lost when downgrading to 1.12.2">
</div>

```python
report = json.loads(build.convert_to_data_version(1343, build.canonical_data_version()))
lost = [e["path"] for e in report if e["severity"] == "loss"]   # what 1.12.2 cannot represent
```
