# Block entities, entities, and NBT

## Block entities, entities, and NBT


Blocks carry NBT, and the schematic holds full block entities and entities,
round-tripped as SNBT, so a chest keeps its loot table and a spawner its mob. A
vault of them: chests, barrels, dyed shulker boxes, a caged spawner, and brewing
and enchanting furniture, every one an NBT carrier:

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/block-entities.png" width="620" alt="An aerial view of a stone-brick vault lined with chests, barrels, dyed shulker boxes, furnaces, a caged spawner, and an enchanting table">
</div>

```python
# Common contents use concise block-string shorthands:
s.set_block(0, 0, 0,
    "minecraft:chest{items=[diamond*64,emerald*12]}")
s.set_block(1, 0, 0,
    "minecraft:barrel{signal=13,item=diamond}")
s.set_block(2, 0, 0,
    "minecraft:jukebox{record=pigstep}")

# A chest with contents, set straight from SNBT:
s.set_block_entity(3, 0, 0, "minecraft:chest",
    '{Items:[{Slot:0b,id:"minecraft:diamond",Count:3b},'
            '{Slot:1b,id:"minecraft:emerald",Count:5b}]}')
s.get_block_entity_snbt(3, 0, 0)
# → {Items:[{...diamond, Slot:0B, Count:3B}, {...emerald, Slot:1B, Count:5B}]}  (SNBT)

# Entities parse from SNBT too:
s.add_entity_from_snbt('{id:"minecraft:armor_stand",Pos:[0.5d,1.0d,0.5d],Rotation:[0f,0f]}')
s.entity_count()                      # 1
```

Container entries use `item*count`, default to a count of one, and fill slots in
order. Counts must be between 1 and 64 and the entry count cannot exceed the
container's slot count. Bare IDs receive the `minecraft:` namespace; namespaced
mod IDs are preserved. Use raw `Items:[...]` NBT when explicit slot numbers or
additional item data are required. Do not mix `items=` with raw `Items:[...]`, or
`record=` with raw `RecordItem:{...}`; those ambiguous pairs are rejected. For
backward compatibility, raw `Items:[...]` takes precedence when combined with
`signal=`.
