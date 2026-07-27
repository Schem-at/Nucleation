import com.mojang.authlib.GameProfile;
import net.minecraft.SharedConstants;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.gametest.framework.GameTestServer;
import net.minecraft.resources.Identifier;
import net.minecraft.server.Bootstrap;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.packs.repository.PackRepository;
import net.minecraft.server.packs.repository.ServerPacksSource;
import net.minecraft.util.RandomSource;
import net.minecraft.util.Util;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.InteractionResult;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.level.GameType;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.levelgen.structure.templatesystem.StructurePlaceSettings;
import net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate;
import net.minecraft.world.level.storage.LevelStorageSource;
import net.minecraft.world.phys.BlockHitResult;
import net.minecraft.world.phys.Vec3;

import java.util.UUID;

import java.lang.reflect.Method;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.function.BooleanSupplier;

/**
 * Captures a tick-by-tick trace of what the real game does, as ground truth for the
 * Rust engine.
 *
 * <h2>How it drives the game</h2>
 *
 * {@code GameTestMainUtil} hands control to {@link MinecraftServer#spin}, which runs
 * the server's loop to completion — fine for pass/fail, useless for tracing, because
 * there is no point between ticks at which to observe. So this builds the same
 * {@link GameTestServer} by the same recipe and then drives {@code tickServer}
 * itself, one tick at a time.
 *
 * <p>{@code initServer} and {@code tickServer} are {@code protected}, so they are
 * reached reflectively. Ugly but honest; the alternative is bytecode injection, which
 * is far more machinery for the same access. The jar is unobfuscated, so the names
 * are stable and readable.
 *
 * <h2>Why it places the structure itself</h2>
 *
 * An earlier attempt let the gametest framework place the structure and then tried to
 * find it by iterating loaded chunks. That never worked — and it was the wrong shape
 * regardless. A tracer should control its scenario: here the structure is placed at a
 * known origin, so the region to observe is known exactly rather than discovered.
 *
 * <h2>What it records, and what it does not</h2>
 *
 * Block changes per tick, in a deterministic scan order. It does <em>not</em>
 * attribute an event to the tick phase it happened in, because observing between
 * ticks cannot see inside one. Events therefore carry the {@code tick_end} sentinel
 * phase. Per-phase attribution needs interception within the tick and is deliberately
 * left for later rather than guessed at — a trace that invented phases would be worse
 * than one that admits what it does not know.
 */
public final class TraceCapture {
    private TraceCapture() {}

    /** Sentinel: we observe between ticks, so the true phase is unknown. */
    private static final String PHASE_TICK_END = "tick_end";

    /** Where the structure is placed. Arbitrary but fixed, so traces are comparable. */
    private static BlockPos ORIGIN = new BlockPos(0, 64, 0);

    /** Blocks of slack around the structure, to catch anything it pushes outward. */
    private static final int MARGIN = 4;

    public static void main(String[] args) throws Exception {
        String universe = arg(args, "--universe", "work/trace-universe");
        String structureId = arg(args, "--structure", "nucleation:torch_inverts");
        int maxTicks = Integer.parseInt(arg(args, "--max-ticks", "40"));
        Path out = Path.of(arg(args, "--out", "work/trace.json"));
        String dumpPlaced = arg(args, "--dump-placed", null);
        String watchAt = arg(args, "--watch", null);
        String probeAt = arg(args, "--probe", null);
        int probeTick = Integer.parseInt(arg(args, "--probe-tick", "-1"));
        String probePush = arg(args, "--probe-push", null);

        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        Util.startTimerHackThread();

        LevelStorageSource storage = LevelStorageSource.createDefault(Paths.get(universe));
        LevelStorageSource.LevelStorageAccess access = storage.createAccess("gametestworld");
        PackRepository packs = ServerPacksSource.createPackRepository(access);

        // An empty test selection: we want the world and its datapacks, not the
        // framework's own placement or scheduling.
        GameTestServer server = GameTestServer.create(
                Thread.currentThread(), access, packs, Optional.empty(), false, 1);

        Method initServer = MinecraftServer.class.getDeclaredMethod("initServer");
        Method tickServer = MinecraftServer.class.getDeclaredMethod(
                "tickServer", BooleanSupplier.class);
        // runServer() does more than tickServer per iteration: it also drains the
        // server task queue. Chunk entity loading is dispatched through that queue,
        // and until it completes LevelTicks refuses to run a chunk's scheduled
        // ticks (it gates on isPositionTickingWithEntitiesLoaded). Ticking without
        // pumping tasks therefore looks exactly like redstone that ignores input.
        Method waitUntilNextTick = MinecraftServer.class
                .getDeclaredMethod("waitUntilNextTick");
        initServer.setAccessible(true);
        tickServer.setAccessible(true);
        waitUntilNextTick.setAccessible(true);
        initServer.invoke(server);

        ServerLevel level = server.overworld();

        // `--in-world x0,y0,z0,x1,y1,z1` records a build *where it already
        // stands*, with no clearing and no placement.
        //
        // Every paste disturbs a machine: placeInWorld recomputes LOCKED on
        // repeaters, recomputes wire connection shapes, and loads block-entity
        // NBT only after the block write. A door that was built in place and
        // left latched cannot be reproduced by stamping its blocks down, no
        // matter how faithful the block data is — which is why a captured door
        // can stall in a way the real one never does. In this mode the game
        // simply keeps ticking the world it loaded.
        String inWorld = arg(args, "--in-world", null);
        BlockPos worldMin = null, worldMax = null;
        if (inWorld != null) {
            String[] b = inWorld.split(",");
            worldMin = new BlockPos(Integer.parseInt(b[0].trim()),
                    Integer.parseInt(b[1].trim()), Integer.parseInt(b[2].trim()));
            worldMax = new BlockPos(Integer.parseInt(b[3].trim()),
                    Integer.parseInt(b[4].trim()), Integer.parseInt(b[5].trim()));
        }

        // Anchor to spawn: those chunks are held at a ticking level by the
        // server, whereas a far-off forced chunk loads but never activates its
        // tick containers.
        BlockPos spawn = level.getRespawnData().pos();
        ORIGIN = inWorld != null ? worldMin : new BlockPos(spawn.getX(), 100, spawn.getZ());
        System.out.printf("  origin: %s%n", ORIGIN);

        StructureTemplate template = inWorld != null
                ? server.getStructureManager().get(Identifier.parse(structureId)).orElse(null)
                : server.getStructureManager()
                        .get(Identifier.parse(structureId))
                        .orElseThrow(() -> new IllegalStateException(
                                "no such structure: " + structureId
                                        + " (is the datapack in <universe>/gametestworld/datapacks?)"));

        net.minecraft.core.Vec3i size = template != null
                ? template.getSize()
                : new net.minecraft.core.Vec3i(0, 0, 0);
        BlockPos min = inWorld != null
                ? worldMin
                : ORIGIN.offset(-MARGIN, -MARGIN, -MARGIN);
        BlockPos max = inWorld != null
                ? worldMax
                : ORIGIN.offset(size.getX() + MARGIN, size.getY() + MARGIN, size.getZ() + MARGIN);

        // The region must *simulate*, not merely be loaded. setChunkForced alone
        // keeps chunks in memory but does not raise them to a ticking level, so
        // scheduled block ticks never fire — which looks exactly like a torch that
        // ignores its input, because a torch changes only on a scheduled tick.
        // Dust hid this: it settles synchronously on neighbour updates and works
        // either way. PLAYER_SIMULATION is the ticket that grants ticking.
        // Ticket *every* chunk the region touches, not just the one under ORIGIN.
        // A ticket's radius degrades outward: the chunk at the edge of the radius
        // is loaded but not ticking. A build that straddles a chunk border — and
        // at seventeen blocks wide this one does — then has half of itself in a
        // chunk whose scheduled ticks are never collected.
        // `chunks` is what must tick — the ones the recording actually covers.
        // The ring around them is ticketed too, so the recorded chunks sit
        // inside the ticket radius rather than on its edge, but a margin chunk
        // that never reaches entity-ticking is not a problem worth failing on.
        var chunks = new java.util.ArrayList<net.minecraft.world.level.ChunkPos>();
        for (int cx = (min.getX() >> 4) - 1; cx <= (max.getX() >> 4) + 1; cx++) {
            for (int cz = (min.getZ() >> 4) - 1; cz <= (max.getZ() >> 4) + 1; cz++) {
                var chunk = new net.minecraft.world.level.ChunkPos(cx, cz);
                boolean covered = cx >= (min.getX() >> 4) && cx <= (max.getX() >> 4)
                        && cz >= (min.getZ() >> 4) && cz <= (max.getZ() >> 4);
                if (covered) {
                    chunks.add(chunk);
                }
                level.getChunkSource().addTicketWithRadius(
                        net.minecraft.server.level.TicketType.PLAYER_SIMULATION, chunk, 2);
                level.getChunkSource().addTicketWithRadius(
                        net.minecraft.server.level.TicketType.FORCED, chunk, 2);
                level.setChunkForced(cx, cz, true);
            }
        }
        // Entity loading is asynchronous, and LevelTicks will not run a chunk's
        // scheduled ticks until its entities are loaded — `ServerLevel` passes
        // `isPositionTickingWithEntitiesLoaded` as the collector's predicate, so
        // ticks in a not-yet-ready chunk sit in the queue, overdue, until it is.
        // Wait for every chunk rather than guessing a tick count: waiting on only
        // the centre one let a neighbouring chunk finish loading twenty-two ticks
        // into a recording and dump its whole backlog in one tick, which reads as
        // a door that deliberately sequences its two halves.
        int warmup = 0;
        while (warmup < 600 && !chunks.stream()
                .allMatch(c -> level.isPositionTickingWithEntitiesLoaded(c.pack()))) {
            tickServer.invoke(server, (BooleanSupplier) () -> true);
            waitUntilNextTick.invoke(server);
            warmup++;
        }
        long ready = chunks.stream()
                .filter(c -> level.isPositionTickingWithEntitiesLoaded(c.pack())).count();
        System.out.printf("  warmup ticks until entity-ticking: %d (%d/%d chunks ready)%n",
                warmup, ready, chunks.size());
        if (ready < chunks.size()) {
            throw new IllegalStateException(
                    "only " + ready + " of " + chunks.size() + " chunks are entity-ticking; "
                            + "the recording would stall and then catch up mid-run");
        }

        if (inWorld == null) {
            for (BlockPos pos : BlockPos.betweenClosed(min, max)) {
                level.setBlock(pos, Blocks.AIR.defaultBlockState(), 2);
            }
        }

        // --known-shape places with StructurePlaceSettings.knownShape, which skips
        // the final update pass — the vanilla-supported "quiet" placement. Without
        // the pass, observers receive no placement shape updates and do not pulse.
        // Note this is not a freeze: per-block onPlace still runs, so e.g. a
        // quasi-connected piston still notices its power source.
        StructurePlaceSettings settings = new StructurePlaceSettings();
        if (hasFlag(args, "--known-shape")) {
            settings.setKnownShape(true);
            System.out.println("  placement: known-shape (quiet, no update pass)");
        }
        if (inWorld == null && !template.placeInWorld(level, ORIGIN, ORIGIN, settings,
                RandomSource.create(0), 3)) {
            throw new IllegalStateException("failed to place " + structureId);
        }

        // Redstone settles synchronously while the structure is placed, so a trace
        // taken from here alone records nothing. To observe propagation we have to
        // disturb the settled state: --break removes a block (typically the power
        // source) and the ticks that follow are the circuit's response.
        // Decisive diagnostic: a region that is loaded but not in block-ticking
        // range looks exactly like a circuit that ignores its input.
        System.out.printf("  block-ticking chunks: %d/%d%n",
                chunks.stream().filter(c -> level.shouldTickBlocksAt(c.pack())).count(),
                chunks.size());
        System.out.printf("  pending block ticks: %d%n", level.getBlockTicks().count());
        System.out.printf("  runs normally: %s%n",
                level.tickRateManager().runsNormally());
        // LevelTicks gates on this, not on block-ticking range: a chunk can be
        // block-tickable yet have no active tick container if its entities are
        // not loaded, and then scheduled ticks sit pending forever.
        System.out.printf("  entity-ticking chunks: %d/%d%n",
                chunks.stream()
                        .filter(c -> level.isPositionTickingWithEntitiesLoaded(c.pack())).count(),
                chunks.size());

        List<String> ticks = new ArrayList<>();
        List<String> queues = new ArrayList<>();
        // The tick numbering in this file is relative; a scheduled tick's
        // trigger time is absolute. Recording the offset lets a reader turn one
        // into the other without guessing.
        long startGameTime = level.getGameTime();
        Map<BlockPos, String> previous = snapshot(level, min, max);
        // The world exactly as placement left it: the ground truth for
        // comparing an engine's settle against the game's.
        if (dumpPlaced != null) {
            java.util.List<BlockPos> keys = new java.util.ArrayList<>(previous.keySet());
            keys.sort(java.util.Comparator
                    .comparingInt((BlockPos k) -> k.getY())
                    .thenComparingInt((BlockPos k) -> k.getZ())
                    .thenComparingInt((BlockPos k) -> k.getX()));
            StringBuilder placed = new StringBuilder();
            for (BlockPos key : keys) {
                placed.append(key.getX() - ORIGIN.getX()).append(' ')
                      .append(key.getY() - ORIGIN.getY()).append(' ')
                      .append(key.getZ() - ORIGIN.getZ()).append(' ')
                      .append(previous.get(key)).append('\n');
            }
            java.nio.file.Files.writeString(Path.of(dumpPlaced), placed.toString());
        }
        // Ask the game itself what a position sees: the six weak signals a
        // block reads, plus the strong signal into each neighbour. This is
        // ground truth for "which neighbour is powering this thing".
        if (probeAt != null && probeTick < 0) {
            String[] pp = probeAt.split(",");
            BlockPos probe = ORIGIN.offset(Integer.parseInt(pp[0].trim()),
                    Integer.parseInt(pp[1].trim()), Integer.parseInt(pp[2].trim()));
            System.out.println("PROBE " + probeAt + " state=" + level.getBlockState(probe));
            System.out.println("  hasNeighborSignal=" + level.hasNeighborSignal(probe)
                    + " bestNeighborSignal=" + level.getBestNeighborSignal(probe));
            for (net.minecraft.core.Direction dir : net.minecraft.core.Direction.values()) {
                BlockPos n = probe.relative(dir);
                System.out.println("  " + dir + " " + level.getBlockState(n)
                        + "\n      getSignal=" + level.getSignal(n, dir)
                        + " directSignalTo=" + level.getDirectSignalTo(n)
                        + " conductor=" + level.getBlockState(n).isRedstoneConductor(level, n));
            }
        }
        Map<BlockPos, String[]> previousInv = snapshotContainers(level, min, max);
        // --entities: also diff item entities per tick. Opt-in because RNG-fed
        // spawns (dispensers) make trajectories sample-specific; deterministic
        // captures author their items in the structure's entity list.
        boolean captureEntities = hasFlag(args, "--entities");
        Map<Integer, double[]> previousEnt =
                captureEntities ? snapshotItems(level, min, max) : new HashMap<>();

        // --pulse places a power source at a position, holds it for --pulse-ticks,
        // then removes it. Short pulses are how several piston behaviours are
        // provoked, and they cannot be produced by --break alone.
        String pulseAt = arg(args, "--pulse", null);
        int pulseTicks = Integer.parseInt(arg(args, "--pulse-ticks", "1"));
        int pulsePeriod = Integer.parseInt(arg(args, "--pulse-period", "0"));
        BlockPos pulsePos = null;
        if (pulseAt != null) {
            String[] pp = pulseAt.split(",");
            pulsePos = ORIGIN.offset(Integer.parseInt(pp[0].trim()),
                    Integer.parseInt(pp[1].trim()), Integer.parseInt(pp[2].trim()));
            level.setBlock(pulsePos, Blocks.REDSTONE_BLOCK.defaultBlockState(), 3);
            System.out.printf("  pulse: powered %s for %d tick(s)%n", pulseAt, pulseTicks);
        }
        final BlockPos pulseTarget = pulsePos;

        // --use right-clicks a block with an empty hand, on a chosen tick boundary.
        // This is the "manual" in a manual engine: a note block does nothing until a
        // player clicks it. The sequence below is GameTestHelper.useBlock verbatim —
        // useItemOn first, falling through to useWithoutItem on
        // TryEmptyHandInteraction — so the capture exercises the same code path a
        // real click does.
        String useAt = arg(args, "--use", null);
        // --use-tick accepts a comma list, so one capture can run a full
        // close/open cycle rather than a single toggle.
        java.util.List<Integer> useTicks = new java.util.ArrayList<>();
        for (String piece : arg(args, "--use-tick", "0").split(",")) {
            useTicks.add(Integer.parseInt(piece.trim()));
        }
        int useTick = useTicks.get(0);
        BlockPos usePos = null;
        if (useAt != null) {
            String[] up = useAt.split(",");
            usePos = ORIGIN.offset(Integer.parseInt(up[0].trim()),
                    Integer.parseInt(up[1].trim()), Integer.parseInt(up[2].trim()));
            System.out.printf("  use: clicking %s before tick %d%n", useAt, useTick);
        }
        final BlockPos useTarget = usePos;

        String breakAt = arg(args, "--break", null);
        if (breakAt != null) {
            String[] parts = breakAt.split(",");
            BlockPos target = ORIGIN.offset(
                    Integer.parseInt(parts[0].trim()),
                    Integer.parseInt(parts[1].trim()),
                    Integer.parseInt(parts[2].trim()));
            level.setBlock(target, Blocks.AIR.defaultBlockState(), 3);
            System.out.printf("  actuated: broke %s%n", breakAt);
        }

        // The state immediately after placement, before any tick. Printed because
        // "no changes" is ambiguous: it means either nothing happened, or everything
        // settled during placement. Only the initial state distinguishes them.
        for (BlockPos pos : BlockPos.betweenClosed(min, max)) {
            String state = previous.get(pos);
            if (state != null && !state.startsWith("minecraft:air")) {
                System.out.printf("  initial [%d,%d,%d] %s%n",
                        pos.getX() - ORIGIN.getX(), pos.getY() - ORIGIN.getY(),
                        pos.getZ() - ORIGIN.getZ(), state);
            }
        }

        installBlockEventLog(level);
        if (watchAt != null) {
            String[] wp = watchAt.split(",");
            WATCH = ORIGIN.offset(Integer.parseInt(wp[0].trim()),
                    Integer.parseInt(wp[1].trim()), Integer.parseInt(wp[2].trim()));
            System.out.printf("  watching %s%n", watchAt);
        }
        installNotifyLog(level);
        for (int tick = 0; tick < maxTicks; tick++) {
            EVENT_LOG.clear();
            NOTIFY_LOG.clear();
            if (useTarget != null && useTicks.contains(tick)) {
                useBlock(level, useTarget);
            }
            if (pulseTarget != null) {
                if (pulsePeriod > 0) {
                    // Square wave: drives a component hard enough to provoke
                    // rate limits such as redstone-torch burnout.
                    boolean on = ((tick / pulsePeriod) % 2) == 0;
                    level.setBlock(pulseTarget, on
                            ? Blocks.REDSTONE_BLOCK.defaultBlockState()
                            : Blocks.AIR.defaultBlockState(), 3);
                } else if (tick == pulseTicks) {
                    level.setBlock(pulseTarget, Blocks.AIR.defaultBlockState(), 3);
                }
            }
            String queuedBefore = queueDump(level);
            tickServer.invoke(server, (BooleanSupplier) () -> true);
            waitUntilNextTick.invoke(server);
            if (tick < 3) {
                System.out.printf("    t%d gameTime=%d pending=%d%n",
                        tick, level.getGameTime(), level.getBlockTicks().count());
            }

            Map<BlockPos, String> current = snapshot(level, min, max);
            Map<BlockPos, String[]> currentInv = snapshotContainers(level, min, max);
            List<String> events = new ArrayList<>();
            // The engine's own queues, before and after the tick ran. A snapshot
            // diff cannot see inside a tick, but the queues bracket it: what the
            // game *planned* to do this tick, and what it left for the next one.
            // That is enough to place an event in a phase without inventing one.
            queues.add(String.format(
                    "    {\"tick\": %d, \"before\": %s, \"after\": %s, \"log\": [%s], \"notify\": [%s]}",
                    tick, queuedBefore, queueDump(level), String.join(", ", EVENT_LOG),
                    String.join(", ", NOTIFY_LOG)));
            for (BlockPos pos : BlockPos.betweenClosed(min, max)) {
                String was = previous.get(pos);
                String now = current.get(pos);
                if (!java.util.Objects.equals(was, now)) {
                    events.add(blockChanged(pos, was, now));
                }
                // Container contents are invisible to the block diff — a hopper
                // transfer changes only block-entity NBT — so containers are
                // diffed slot by slot.
                String[] invWas = previousInv.get(pos);
                String[] invNow = currentInv.get(pos);
                int slots = Math.max(invWas == null ? 0 : invWas.length,
                        invNow == null ? 0 : invNow.length);
                for (int slot = 0; slot < slots; slot++) {
                    String slotWas = invWas != null && slot < invWas.length ? invWas[slot] : "";
                    String slotNow = invNow != null && slot < invNow.length ? invNow[slot] : "";
                    if (!slotWas.equals(slotNow)) {
                        events.add(String.format(
                                "        {\"phase\": \"%s\", \"kind\": \"inventory_changed\", "
                                        + "\"pos\": [%d, %d, %d], \"slot\": %d, "
                                        + "\"from\": \"%s\", \"to\": \"%s\"}",
                                PHASE_TICK_END, pos.getX() - ORIGIN.getX(),
                                pos.getY() - ORIGIN.getY(), pos.getZ() - ORIGIN.getZ(),
                                slot, slotWas, slotNow));
                    }
                }
            }
            if (probePush != null && probeTick == tick) {
                String[] pp = probePush.split(",");
                BlockPos probe = ORIGIN.offset(Integer.parseInt(pp[0].trim()),
                        Integer.parseInt(pp[1].trim()), Integer.parseInt(pp[2].trim()));
                net.minecraft.world.level.block.state.BlockState ps = level.getBlockState(probe);
                System.out.println("PUSH@" + tick + " " + probePush + " state=" + ps);
                if (ps.getBlock() instanceof net.minecraft.world.level.block.piston.PistonBaseBlock) {
                    net.minecraft.core.Direction facing = ps.getValue(
                            net.minecraft.world.level.block.piston.PistonHeadBlock.FACING);
                    net.minecraft.world.level.block.piston.PistonStructureResolver resolver =
                            new net.minecraft.world.level.block.piston.PistonStructureResolver(
                                    level, probe, facing, true);
                    boolean ok = resolver.resolve();
                    System.out.println("  resolve=" + ok + " toPush=" + resolver.getToPush().size()
                            + " toDestroy=" + resolver.getToDestroy().size());
                    for (BlockPos bp : resolver.getToPush()) {
                        System.out.println("    push " + (bp.getX() - ORIGIN.getX()) + ","
                                + (bp.getY() - ORIGIN.getY()) + "," + (bp.getZ() - ORIGIN.getZ())
                                + " " + level.getBlockState(bp));
                    }
                }
            }
            if (probeAt != null && probeTick == tick) {
                String[] pp = probeAt.split(",");
                BlockPos probe = ORIGIN.offset(Integer.parseInt(pp[0].trim()),
                        Integer.parseInt(pp[1].trim()), Integer.parseInt(pp[2].trim()));
                System.out.println("PROBE@" + tick + " " + probeAt + " state=" + level.getBlockState(probe));
                System.out.println("  hasNeighborSignal=" + level.hasNeighborSignal(probe)
                        + " bestNeighborSignal=" + level.getBestNeighborSignal(probe));
                for (net.minecraft.core.Direction dir : net.minecraft.core.Direction.values()) {
                    BlockPos n = probe.relative(dir);
                    System.out.println("  " + dir + " " + level.getBlockState(n)
                            + " getSignal=" + level.getSignal(n, dir)
                            + " directTo=" + level.getDirectSignalTo(n)
                            + " cond=" + level.getBlockState(n).isRedstoneConductor(level, n));
                }
            }
            if (captureEntities) {
                Map<Integer, double[]> currentEnt = snapshotItems(level, min, max);
                for (Map.Entry<Integer, double[]> entry : currentEnt.entrySet()) {
                    double[] was = previousEnt.get(entry.getKey());
                    double[] now = entry.getValue();
                    boolean moved = was == null;
                    if (!moved) {
                        for (int i = 0; i < 3; i++) {
                            if (Math.abs(was[i] - now[i]) > 1.0e-9) {
                                moved = true;
                            }
                        }
                    }
                    if (moved) {
                        events.add(String.format(
                                "        {\"phase\": \"%s\", \"kind\": \"entity_moved\", "
                                        + "\"id\": %d, \"entity_type\": \"%s\", "
                                        + "\"pos\": [%s, %s, %s], \"velocity\": [%s, %s, %s]}",
                                PHASE_TICK_END, entry.getKey(),
                                ENTITY_TYPES.getOrDefault(entry.getKey(), "minecraft:item"),
                                Double.toString(now[0]), Double.toString(now[1]),
                                Double.toString(now[2]), Double.toString(now[3]),
                                Double.toString(now[4]), Double.toString(now[5])));
                    }
                }
                for (Integer id : previousEnt.keySet()) {
                    if (!currentEnt.containsKey(id)) {
                        events.add(String.format(
                                "        {\"phase\": \"%s\", \"kind\": \"entity_removed\", \"id\": %d}",
                                PHASE_TICK_END, id));
                    }
                }
                previousEnt = currentEnt;
            }
            previous = current;
            previousInv = currentInv;

            if (!events.isEmpty()) {
                ticks.add("    {\n      \"tick\": " + tick + ",\n      \"events\": [\n"
                        + String.join(",\n", events) + "\n      ]\n    }");
            }
        }

        String json = "{\n"
                + "  \"format_version\": 1,\n"
                + "  \"mc_version\": \"" + SharedConstants.getCurrentVersion().name() + "\",\n"
                + "  \"structure\": \"" + structureId + "\",\n"
                // Where local (0,0,0) sits in the game's coordinates. Only the
                // HashSet in updatePowerStrength cares, and it cares absolutely:
                // BlockPos.hashCode runs on the world position, so a build
                // recorded away from the origin hands updateNeighborsAt its
                // seven positions in an order a zero-based replay cannot guess.
                + "  \"origin\": [" + ORIGIN.getX() + ", " + ORIGIN.getY() + ", "
                + ORIGIN.getZ() + "],\n"
                + "  \"detail\": \"normal\",\n"
                + "  \"game_time_at_start\": " + startGameTime + ",\n"
                + "  \"queues\": [\n" + String.join(",\n", queues) + "\n  ],\n"
                + "  \"ticks\": [\n" + String.join(",\n", ticks) + "\n  ]\n}\n";

        Path parent = out.getParent();
        if (parent != null) {
            Files.createDirectories(parent);
        }
        Files.writeString(out, json);
        System.out.printf("captured %d tick(s) with changes -> %s%n", ticks.size(), out);

        server.halt(false);

        // Delete the world. Each run creates a full Minecraft save — all three
        // dimensions with region files — and a session that captures a dozen
        // traces will fill a disk. Regenerating it costs a few seconds.
        deleteRecursively(Paths.get(universe));

        System.exit(0);
    }

    /**
     * Right-click `pos` with an empty main hand, exactly as
     * {@code GameTestHelper.useBlock} does it.
     *
     * <p>The player is the same construction as {@code makeMockPlayer}: an anonymous
     * {@link Player} subclass — the only abstract method is {@code gameMode()}, and
     * {@code isClientAuthoritative()} is overridden to keep the server authoritative,
     * matching the framework's mock. The player is never added to the level; the
     * framework's mock isn't either, and the vanilla use path tolerates that.
     */
    private static void useBlock(ServerLevel level, BlockPos pos) {
        Player player = new Player(level,
                new GameProfile(UUID.randomUUID(), "trace-mock-player")) {
            @Override
            public GameType gameMode() {
                return GameType.CREATIVE;
            }

            @Override
            public boolean isClientAuthoritative() {
                return false;
            }
        };
        BlockState state = level.getBlockState(pos);
        InteractionHand hand = InteractionHand.MAIN_HAND;
        BlockHitResult hit = new BlockHitResult(
                Vec3.atCenterOf(pos), Direction.NORTH, pos, true);
        InteractionResult result =
                state.useItemOn(player.getItemInHand(hand), level, player, hand, hit);
        if (result.consumesAction()) {
            return;
        }
        if (result instanceof InteractionResult.TryEmptyHandInteraction) {
            state.useWithoutItem(level, player, hit);
        }
        // GameTestHelper would fall through to ItemStack.useOn here; the hand is
        // empty by construction, so there is nothing to use.
    }

    /**
     * Every item entity in the box: id -> [x, y, z, vx, vy, vz].
     *
     * <p>Positions relative to ORIGIN, so they compare against the engine's
     * structure-relative coordinates directly.
     */
    private static Map<Integer, double[]> snapshotItems(
            ServerLevel level, BlockPos min, BlockPos max) {
        Map<Integer, double[]> items = new java.util.TreeMap<>();
        net.minecraft.world.phys.AABB box = new net.minecraft.world.phys.AABB(
                min.getX(), min.getY(), min.getZ(),
                max.getX() + 1, max.getY() + 1, max.getZ() + 1);
        for (net.minecraft.world.entity.Entity entity : level.getEntitiesOfClass(
                net.minecraft.world.entity.Entity.class, box)) {
            boolean tracked = entity instanceof net.minecraft.world.entity.item.ItemEntity
                    || entity instanceof net.minecraft.world.entity.vehicle.minecart.AbstractMinecart;
            if (!tracked) {
                continue;
            }
            net.minecraft.world.phys.Vec3 velocity = entity.getDeltaMovement();
            items.put(entity.getId(), new double[] {
                    entity.getX() - ORIGIN.getX(), entity.getY() - ORIGIN.getY(),
                    entity.getZ() - ORIGIN.getZ(),
                    velocity.x, velocity.y, velocity.z});
            ENTITY_TYPES.put(entity.getId(),
                    net.minecraft.core.registries.BuiltInRegistries.ENTITY_TYPE
                            .getKey(entity.getType()).toString());
        }
        return items;
    }

    /** Entity id -> registry type name, filled by snapshotItems. */
    private static final Map<Integer, String> ENTITY_TYPES = new HashMap<>();

    /**
     * Container contents for every container in the box, one string per slot.
     *
     * <p>Slots render as {@code "<count>x <id>"} or {@code ""}, matching the
     * engine's rendering, so the two sides diff string-for-string.
     */
    private static Map<BlockPos, String[]> snapshotContainers(
            ServerLevel level, BlockPos min, BlockPos max) {
        Map<BlockPos, String[]> containers = new HashMap<>();
        for (BlockPos pos : BlockPos.betweenClosed(min, max)) {
            if (level.getBlockEntity(pos)
                    instanceof net.minecraft.world.level.block.entity.BaseContainerBlockEntity container) {
                String[] slots = new String[container.getContainerSize()];
                for (int slot = 0; slot < slots.length; slot++) {
                    net.minecraft.world.item.ItemStack stack = container.getItem(slot);
                    slots[slot] = stack.isEmpty()
                            ? ""
                            : stack.getCount() + "x "
                                    + net.minecraft.core.registries.BuiltInRegistries.ITEM
                                            .getKey(stack.getItem());
                }
                containers.put(pos.immutable(), slots);
            }
        }
        return containers;
    }

    /** Descriptors for every position in the box, so comparison is order-independent. */
    /**
     * The level's pending work, read straight off its queues by reflection:
     * block events (in queue order, which is run order), the events refused last
     * tick and rescheduled, and the scheduled block ticks due next.
     *
     * Captured once per tick either side of the tick itself, this answers the
     * ordering questions a between-ticks snapshot cannot — whether a piston ever
     * had an event queued at all, and in what order two diodes will fire.
     */
    @SuppressWarnings("unchecked")
    /// Every `blockEvent` call and every `runBlockEvents` poll, in order.
    ///
    /// The before/after queue dumps bracket a tick but cannot see inside it, and
    /// the order block events are *queued* in during a tick is what decides which
    /// of two racing pistons wins. It is otherwise unobservable: the queue is
    /// filled and drained entirely within one `tickServer` call, and freezing the
    /// chunk to stop the drain also stops the scheduled ticks that do the filling.
    ///
    /// `ServerLevel.blockEvents` is a plain `ObjectLinkedOpenHashSet` on the
    /// classpath and `blockEvent()` does nothing but `add` to it, so a subclass
    /// that logs and delegates records the true order without changing any of it.
    /// Reflection can write it because the field is final but not static.
    private static final List<String> EVENT_LOG = new ArrayList<>();

    private static final class LoggingBlockEvents
            extends it.unimi.dsi.fastutil.objects.ObjectLinkedOpenHashSet<
                    net.minecraft.world.level.BlockEventData> {
        @Override
        public boolean add(net.minecraft.world.level.BlockEventData e) {
            boolean fresh = super.add(e);
            // A repeat add is dropped by the set and keeps its original place,
            // which is exactly the detail that decides these races — so record
            // whether it took.
            String line = String.format(
                    "{\"kind\": \"%s\", \"pos\": [%d, %d, %d], \"id\": %d, \"param\": %d}",
                    fresh ? "queue" : "requeue",
                    e.pos().getX() - ORIGIN.getX(), e.pos().getY() - ORIGIN.getY(),
                    e.pos().getZ() - ORIGIN.getZ(), e.paramA(), e.paramB());
            EVENT_LOG.add(line);
            if (NOTIFY_LOG.size() < NOTIFY_LIMIT) NOTIFY_LOG.add(line);
            return fresh;
        }

        @Override
        public net.minecraft.world.level.BlockEventData removeFirst() {
            net.minecraft.world.level.BlockEventData e = super.removeFirst();
            String line = String.format(
                    "{\"kind\": \"run\", \"pos\": [%d, %d, %d], \"id\": %d, \"param\": %d}",
                    e.pos().getX() - ORIGIN.getX(), e.pos().getY() - ORIGIN.getY(),
                    e.pos().getZ() - ORIGIN.getZ(), e.paramA(), e.paramB());
            EVENT_LOG.add(line);
            if (NOTIFY_LOG.size() < NOTIFY_LIMIT) NOTIFY_LOG.add(line);
            return e;
        }
    }

    @SuppressWarnings("unchecked")
    private static void installBlockEventLog(ServerLevel level) {
        try {
            java.lang.reflect.Field f = ServerLevel.class.getDeclaredField("blockEvents");
            f.setAccessible(true);
            LoggingBlockEvents logging = new LoggingBlockEvents();
            logging.addAll((java.util.Collection<net.minecraft.world.level.BlockEventData>) f.get(level));
            EVENT_LOG.clear();
            f.set(level, logging);
        } catch (ReflectiveOperationException e) {
            throw new IllegalStateException("could not instrument blockEvents", e);
        }
    }

    /// Every neighbour update the game *asks for*, in order.
    ///
    /// The block-event log says which events ran and says nothing about the
    /// notifications between them — and those decide the order events get
    /// queued in, so a divergence hunt with only events keeps landing on
    /// symptoms. `CollectingNeighborUpdater` is a public non-final class whose
    /// entry points are all overridable, so a subclass that logs and calls
    /// super records the sequence without altering it. The field holding it on
    /// `Level` is final but not static, which reflection can still write.
    ///
    /// Its own `setDebugListener` looks like the intended hook and never fires
    /// here; overriding the entry points does.
    ///
    /// Granularity is the *call*, not the individual notification:
    /// `updateNeighborsAtExceptFromFacing` is one line covering six
    /// neighbours, which is exactly how the engine models an update entry.
    private static final List<String> NOTIFY_LOG = new ArrayList<>();

    /// Beyond this a tick's notifications stop being worth the file size.
    private static final int NOTIFY_LIMIT = 40000;

    private static void note(String kind, BlockPos pos) {
        if (NOTIFY_LOG.size() < NOTIFY_LIMIT) {
            NOTIFY_LOG.add(String.format("{\"kind\": \"%s\", \"pos\": [%d, %d, %d]}",
                    kind, pos.getX() - ORIGIN.getX(), pos.getY() - ORIGIN.getY(),
                    pos.getZ() - ORIGIN.getZ()));
        }
    }

    /// `--watch x,y,z` — sample a position at every notification and report it
    /// whenever it changes.
    ///
    /// Mid-tick state is otherwise unobservable: a snapshot capture only sees
    /// the end of a tick, and a probe runs between ticks. But the updater holds
    /// the level and is called throughout the cascade, so sampling from inside
    /// it gives a view of the world *as the tick runs* — which is where every
    /// remaining divergence has been hiding.
    private static BlockPos WATCH = null;
    private static String watchLast = null;

    private static final class LoggingUpdater
            extends net.minecraft.world.level.redstone.CollectingNeighborUpdater {
        private final net.minecraft.world.level.Level level;

        LoggingUpdater(net.minecraft.world.level.Level level, int max) {
            super(level, max);
            this.level = level;
        }

        private void sample() {
            if (WATCH == null) {
                return;
            }
            // Evaluated the way PistonBaseBlock.getNeighborSignal does, because
            // hasNeighborSignal excludes quasi-connectivity and a piston's power
            // usually *is* quasi-connectivity.
            StringBuilder src = new StringBuilder();
            for (net.minecraft.core.Direction d : net.minecraft.core.Direction.values()) {
                int sig = level.getSignal(WATCH.relative(d), d);
                if (sig > 0) src.append(String.format(" %s=%d", d, sig));
            }
            BlockPos above = WATCH.above();
            for (net.minecraft.core.Direction d : net.minecraft.core.Direction.values()) {
                if (d == net.minecraft.core.Direction.DOWN) continue;
                int sig = level.getSignal(above.relative(d), d);
                if (sig > 0) src.append(String.format(" qc:%s=%d", d, sig));
            }
            String now = String.format("%s powered=%s best=%d  sources:%s",
                    level.getBlockState(WATCH), level.hasNeighborSignal(WATCH),
                    level.getBestNeighborSignal(WATCH),
                    src.length() == 0 ? " none" : src.toString());
            if (!now.equals(watchLast)) {
                watchLast = now;
                System.out.printf("  WATCH@%d %s%n", NOTIFY_LOG.size(), now);
            }
        }

        @Override
        public void shapeUpdate(Direction dir, BlockState state, BlockPos pos, BlockPos neighborPos,
                int flags, int recursion) {
            note("shape", pos);
            sample();
            super.shapeUpdate(dir, state, pos, neighborPos, flags, recursion);
        }

        @Override
        public void neighborChanged(BlockPos pos, net.minecraft.world.level.block.Block block,
                net.minecraft.world.level.redstone.Orientation orientation) {
            note("neighbor", pos);
            sample();
            super.neighborChanged(pos, block, orientation);
        }

        @Override
        public void neighborChanged(BlockState state, BlockPos pos,
                net.minecraft.world.level.block.Block block,
                net.minecraft.world.level.redstone.Orientation orientation, boolean movedByPiston) {
            note("neighbor", pos);
            sample();
            super.neighborChanged(state, pos, block, orientation, movedByPiston);
        }

        @Override
        public void updateNeighborsAtExceptFromFacing(BlockPos pos,
                net.minecraft.world.level.block.Block block, Direction skip,
                net.minecraft.world.level.redstone.Orientation orientation) {
            note("neighbors_at", pos);
            sample();
            super.updateNeighborsAtExceptFromFacing(pos, block, skip, orientation);
        }
    }

    private static void installNotifyLog(ServerLevel level) {
        try {
            java.lang.reflect.Field f =
                    net.minecraft.world.level.Level.class.getDeclaredField("neighborUpdater");
            f.setAccessible(true);
            java.lang.reflect.Field maxField =
                    net.minecraft.world.level.redstone.CollectingNeighborUpdater.class
                            .getDeclaredField("maxChainedNeighborUpdates");
            maxField.setAccessible(true);
            int max = maxField.getInt(f.get(level));
            f.set(level, new LoggingUpdater(level, max));
            System.out.printf("  notify hook: installed (maxChained=%d)%n", max);
        } catch (ReflectiveOperationException e) {
            throw new IllegalStateException("could not instrument neighborUpdater", e);
        }
    }

    private static String queueDump(ServerLevel level) {
        StringBuilder out = new StringBuilder("{");
        try {
            java.lang.reflect.Field f = ServerLevel.class.getDeclaredField("blockEvents");
            f.setAccessible(true);
            out.append("\"events\": [");
            boolean first = true;
            for (Object raw : (Iterable<Object>) f.get(level)) {
                net.minecraft.world.level.BlockEventData e =
                        (net.minecraft.world.level.BlockEventData) raw;
                if (!first) out.append(", ");
                first = false;
                out.append(String.format("{\"pos\": [%d, %d, %d], \"block\": \"%s\", \"id\": %d, \"param\": %d}",
                        e.pos().getX() - ORIGIN.getX(),
                        e.pos().getY() - ORIGIN.getY(),
                        e.pos().getZ() - ORIGIN.getZ(),
                        net.minecraft.core.registries.BuiltInRegistries.BLOCK.getKey(e.block()), e.paramA(), e.paramB()));
            }
            out.append("]");

            java.lang.reflect.Field r = ServerLevel.class.getDeclaredField("blockEventsToReschedule");
            r.setAccessible(true);
            out.append(", \"rescheduled\": ").append(((java.util.List<?>) r.get(level)).size());
        } catch (ReflectiveOperationException e) {
            out.append("\"error\": \"").append(e.getClass().getSimpleName()).append("\"");
        }
        out.append(", \"block_ticks\": ").append(level.getBlockTicks().count());

        // Every *scheduled* block tick, with the tick it fires on and its
        // priority. Counts alone cannot answer "which component did the game
        // schedule that we did not", which is the question that actually comes
        // up: two engines can agree on the world and disagree on what is
        // pending in it.
        out.append(", \"scheduled\": [");
        try {
            java.lang.reflect.Field cf =
                    net.minecraft.world.ticks.LevelTicks.class.getDeclaredField("allContainers");
            cf.setAccessible(true);
            Object containers = cf.get(level.getBlockTicks());
            java.lang.reflect.Method values = containers.getClass().getMethod("values");
            java.lang.reflect.Field qf =
                    net.minecraft.world.ticks.LevelChunkTicks.class.getDeclaredField("tickQueue");
            qf.setAccessible(true);
            boolean first = true;
            for (Object chunk : (Iterable<Object>) values.invoke(containers)) {
                for (Object raw : (Iterable<Object>) qf.get(chunk)) {
                    net.minecraft.world.ticks.ScheduledTick<?> st =
                            (net.minecraft.world.ticks.ScheduledTick<?>) raw;
                    if (!first) out.append(", ");
                    first = false;
                    out.append(String.format(
                            "{\"pos\": [%d, %d, %d], \"block\": \"%s\", \"at\": %d, \"priority\": \"%s\", \"order\": %d}",
                            st.pos().getX() - ORIGIN.getX(),
                            st.pos().getY() - ORIGIN.getY(),
                            st.pos().getZ() - ORIGIN.getZ(),
                            net.minecraft.core.registries.BuiltInRegistries.BLOCK.getKey(
                                    (net.minecraft.world.level.block.Block) st.type()),
                            st.triggerTick(), st.priority(), st.subTickOrder()));
                }
            }
        } catch (ReflectiveOperationException e) {
            out.append("]").append(", \"scheduled_error\": \"").append(e.getClass().getSimpleName()).append("\"");
            return out.append("}").toString();
        }
        out.append("]");
        return out.append("}").toString();
    }

    private static Map<BlockPos, String> snapshot(ServerLevel level, BlockPos min, BlockPos max) {
        Map<BlockPos, String> blocks = new HashMap<>();
        for (BlockPos pos : BlockPos.betweenClosed(min, max)) {
            blocks.put(pos.immutable(), describe(level.getBlockState(pos)));
        }
        return blocks;
    }

    private static String blockChanged(BlockPos pos, String from, String to) {
        return String.format(
                "        {\"phase\": \"%s\", \"kind\": \"block_changed\", "
                        + "\"pos\": [%d, %d, %d], \"from\": \"%s\", \"to\": \"%s\"}",
                PHASE_TICK_END, pos.getX() - ORIGIN.getX(), pos.getY() - ORIGIN.getY(),
                pos.getZ() - ORIGIN.getZ(),
                from == null ? "minecraft:air" : from,
                to == null ? "minecraft:air" : to);
    }

    /**
     * Render a state as the descriptor the Rust registry interns.
     *
     * <p>{@code BlockState.toString()} renders as {@code Block{minecraft:stone}[k=v]};
     * strip the wrapper so both sides agree on spelling.
     */
    private static String describe(BlockState state) {
        String text = state.toString();
        int open = text.indexOf('{');
        int close = text.indexOf('}');
        if (open >= 0 && close > open) {
            return text.substring(open + 1, close) + text.substring(close + 1);
        }
        return text;
    }

    /** Remove a directory tree, ignoring anything that resists. */
    private static void deleteRecursively(Path root) {
        if (!Files.exists(root)) {
            return;
        }
        try (var walk = Files.walk(root)) {
            walk.sorted(java.util.Comparator.reverseOrder()).forEach(path -> {
                try {
                    Files.deleteIfExists(path);
                } catch (Exception ignored) {
                    // Best effort: a leftover file is not worth failing a capture.
                }
            });
        } catch (Exception ignored) {
            // Likewise.
        }
    }

    private static boolean hasFlag(String[] args, String name) {
        for (String arg : args) {
            if (arg.equals(name)) {
                return true;
            }
        }
        return false;
    }

    private static String arg(String[] args, String name, String fallback) {
        for (int i = 0; i < args.length - 1; i++) {
            if (args[i].equals(name)) {
                return args[i + 1];
            }
        }
        return fallback;
    }
}
