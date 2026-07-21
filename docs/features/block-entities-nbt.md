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
# A chest with contents, set straight from SNBT:
s.set_block_entity(0, 0, 0, "minecraft:chest",
    '{Items:[{Slot:0b,id:"minecraft:diamond",Count:3b},'
            '{Slot:1b,id:"minecraft:emerald",Count:5b}]}')
s.get_block_entity_snbt(0, 0, 0)
# → {Items:[{...diamond, Slot:0B, Count:3B}, {...emerald, Slot:1B, Count:5B}]}  (SNBT)

# Entities parse from SNBT too:
s.add_entity_from_snbt('{id:"minecraft:armor_stand",Pos:[0.5d,1.0d,0.5d],Rotation:[0f,0f]}')
s.entity_count()                      # 1
```
