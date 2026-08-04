# Parked: tests the engine does not intend to support yet

Self-contained lithium ports (same provenance as `../lithium-litematic/` —
see the `NOTICE.md` there, LGPL-3.0 attribution applies here too) whose pass
condition depends on **mob AI / entity behaviours** mc-tick deliberately does
not simulate: pig/villager/witch AI, frog and goat jumps, llama and lava
pathfinding, turtle-egg trampling.

They are kept runnable on purpose — the day entities grow behaviour, point
the runner here and watch them flip:

```sh
cargo run -p nucleation-cli -- test --path tests/corpus/lithium-litematic-unsupported
```

They are *not* part of the supported corpus run, so a red here never blocks
anything. When re-porting from upstream (`nucleation port`), move these stems
back out of `lithium-litematic/`:

```
ai_baby_pig_follow_parent  ai_johnny_witch_throw_regeneration
ai_villager_hide_in_home   destroy_turtle_egg   destroy_turtle_egg2
frog_jump  goat_jump  llama_pathfinding  pathfinding_avoid_lava
pathfinding_prefer_water_over_lava  pig_dispensed_onto_pressure_plate
```
