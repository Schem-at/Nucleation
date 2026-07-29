# Does 1.21.3 discard non-finite `Motion`? — No.

**Answer: 1.21.3's `Entity.setDeltaMovement(Vec3)` has NO finite guard.** The guard
was introduced in **1.21.11**. The 55x3x3 door's nan carts are genuinely
version-locked; they survived a cold load when the door was built.

Reproduce: `bash tools/gametest/nan-motion-bisect.sh <version>`

## The bytecode

`world_version` of the unpacked 1.21.3 jar is **4082**, matching the save exactly.

```
1.21.3  Entity.setDeltaMovement(Vec3)      == bvk.h(fby)
    0: aload_0
    1: aload_1
    2: putfield  #311   // Field az:Lfby;      <- unconditional store
    5: return

26.2    Entity.setDeltaMovement(Vec3)
    0: aload_1
    1: invokevirtual  Vec3.isFinite:()Z
    4: ifeq  12                                <- skips the store
    7: aload_0
    8: aload_1
    9: putfield  deltaMovement:Lnet/minecraft/world/phys/Vec3;
   12: return
```

`net.minecraft.world.phys.Vec3.isFinite()` **does not exist at all** in 1.21.3 —
it is absent from the class and from the ProGuard mapping.

## Why NaN also survives `Entity.load` in 1.21.3

`Entity.load` (`bvk.g(ux)`, mapped source 2061:2151) does clamp Motion, but not
for NaN. Offsets 30-106 are:

```java
setDeltaMovement(Math.abs(x) > 10.0 ? 0.0 : x, ... same for y, z ...);
```

Compiled as `Math.abs(d); ldc2_w 10.0; dcmpl; ifle <keep>`. **`dcmpl` yields -1
when either operand is NaN**, so `ifle` is taken and the original value is kept.
Verified empirically (`work-1213/nanprobe`, bytecode byte-identical to the game's):

| input | result |
|---|---|
| `NaN` | **`NaN`** (survives) |
| `+Inf` | `0.0` (clamped) |
| `99.0` | `0.0` (clamped) |
| `0.5` | `0.5` |

So infinities are killed but NaN passes through, into
`setDeltaMovement(double,double,double)` -> `new Vec3(...)` ->
the unguarded `setDeltaMovement(Vec3)`.

`Entity.load` in 1.21.3 does contain five `Double.isFinite` calls, but all are on
**position** (-> `IllegalStateException("Entity has invalid position")`) and
**rotation** (-> `"Entity has invalid rotation"`). None touches `deltaMovement`.

## The save satisfies those position/rotation guards

Parsed `tests/samples/55_3x3.zip` -> `entities/*.mca` directly (2 non-empty
region files, `DataVersion 4082`): **22 entities, 6 with non-finite Motion,
0 with non-finite Pos, 0 with non-finite Rotation.** All six carry NaN in
`Motion.z` only, and every `Motion.x` is within the +-10 clamp:

```
furnace_minecart  Pos=[-0.57, 3.0, 19.51]  Motion=[4.28e-59,  0.0,      NaN]
furnace_minecart  Pos=[-0.57, 3.0, 19.49]  Motion=[3.89e-59,  0.0,      NaN]
minecart          Pos=[-1.70, 2.06, 19.50] Motion=[-0.542609, -0.05605, NaN]
minecart          Pos=[-1.69, 2.0,  19.50] Motion=[0.165893,  0.0,      NaN]
minecart          Pos=[-1.70, 2.06, 19.49] Motion=[-0.523454, -0.05605, NaN]
minecart          Pos=[-1.69, 2.0,  19.49] Motion=[0.150492,  0.0,      NaN]
```

Nothing on the 1.21.3 load path throws or sanitises. All six carts load with
`Motion.z == NaN` intact.

## Where the guard appeared: 1.21.11

Bisected across every release from 1.21.3 to 26.2 by parsing each version's
ProGuard mapping (`Vec3.isFinite` presence + the source-line span of
`setDeltaMovement(Vec3)`), then confirmed on the boundary with real bytecode.

| version | `Vec3.isFinite` | `setDeltaMovement(Vec3)` span |
|---|---|---|
| 1.21.3 | absent | 3711:3712 (2 lines) |
| 1.21.4 | absent | 3743:3744 |
| 1.21.5 | absent | 3750:3751 |
| 1.21.6 | absent | 3935:3936 |
| 1.21.7 | absent | 3935:3936 |
| 1.21.8 | absent | 3935:3936 |
| 1.21.9 | absent | 4006:4007 |
| 1.21.10 | absent | 4012:4013 |
| **1.21.11** | **`372:372:boolean isFinite() -> n`** | **4027:4030 (4 lines)** |
| 26.1 - 26.2 | present | (jar unobfuscated, no mappings) |

Boundary confirmed by disassembly:

- **1.21.10** (`world_version 4556`), `Entity` = `cdv`, setter = `k(foh)`:
  `aload_0; aload_1; putfield aY; return` — **no guard**.
- **1.21.11** (`world_version 4671`), `Entity` = `cgk`, setter = `k(ftm)`:
  `aload_1; invokevirtual ftm.n:()Z; ifeq 12; ...; putfield aY` — **guarded**.

## Consequence

The behaviour is version-dependent and an engine modelling this must gate on it:

- **DataVersion <= 4556 (<= 1.21.10)**: `Motion` NaN is preserved through
  `Entity.load`; `+-Inf` and `|v| > 10` are zeroed. Nan carts work from a cold load.
- **DataVersion >= 4671 (>= 1.21.11)**: non-finite `Motion` is silently dropped
  and the previous velocity kept. The door comes apart during warmup.

The 26.2 non-finite count of 0 is therefore correct for 26.2 and says nothing
about 1.21.3.

## Caveats — what was and was not checked

- Verified by **disassembly only**. No 1.21.3 server was booted and the world was
  not loaded live; the "the door works" conclusion is the bytecode's, not a
  runtime observation.
- Only `Entity.setDeltaMovement` / `Entity.load` were audited. `AbstractMinecart`
  overrides and per-tick physics in 1.21.3 were not disassembled, so this proves
  NaN *loads*, not that every downstream tick treats it as the builders expect.
- `dcmpl`-keeps-NaN was confirmed on JDK 25 against bytecode identical to the
  game's; it is specified JVM behaviour, not platform-dependent.
- Only *release* versions were bisected. The guard could have landed in a 1.21.11
  snapshot; the release boundary is 1.21.10 -> 1.21.11.

Working data (gitignorable, ~200 MB): `tools/gametest/work-1213/`,
`tools/gametest/work-bisect/`.
