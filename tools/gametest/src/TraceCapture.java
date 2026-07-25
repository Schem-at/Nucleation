import net.minecraft.SharedConstants;
import net.minecraft.core.BlockPos;
import net.minecraft.gametest.framework.GameTestServer;
import net.minecraft.resources.Identifier;
import net.minecraft.server.Bootstrap;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.packs.repository.PackRepository;
import net.minecraft.server.packs.repository.ServerPacksSource;
import net.minecraft.util.RandomSource;
import net.minecraft.util.Util;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.levelgen.structure.templatesystem.StructurePlaceSettings;
import net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate;
import net.minecraft.world.level.storage.LevelStorageSource;

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

        // Anchor to spawn: those chunks are held at a ticking level by the
        // server, whereas a far-off forced chunk loads but never activates its
        // tick containers.
        BlockPos spawn = level.getRespawnData().pos();
        ORIGIN = new BlockPos(spawn.getX(), 100, spawn.getZ());
        System.out.printf("  origin: %s%n", ORIGIN);

        StructureTemplate template = server.getStructureManager()
                .get(Identifier.parse(structureId))
                .orElseThrow(() -> new IllegalStateException(
                        "no such structure: " + structureId
                                + " (is the datapack in <universe>/gametestworld/datapacks?)"));

        net.minecraft.core.Vec3i size = template.getSize();
        BlockPos min = ORIGIN.offset(-MARGIN, -MARGIN, -MARGIN);
        BlockPos max = ORIGIN.offset(size.getX() + MARGIN, size.getY() + MARGIN,
                size.getZ() + MARGIN);

        // The region must *simulate*, not merely be loaded. setChunkForced alone
        // keeps chunks in memory but does not raise them to a ticking level, so
        // scheduled block ticks never fire — which looks exactly like a torch that
        // ignores its input, because a torch changes only on a scheduled tick.
        // Dust hid this: it settles synchronously on neighbour updates and works
        // either way. PLAYER_SIMULATION is the ticket that grants ticking.
        var centre = new net.minecraft.world.level.ChunkPos(ORIGIN.getX() >> 4, ORIGIN.getZ() >> 4);
        level.getChunkSource().addTicketWithRadius(
                net.minecraft.server.level.TicketType.PLAYER_SIMULATION, centre, 3);
        level.getChunkSource().addTicketWithRadius(
                net.minecraft.server.level.TicketType.FORCED, centre, 3);
        for (int cx = min.getX() >> 4; cx <= max.getX() >> 4; cx++) {
            for (int cz = min.getZ() >> 4; cz <= max.getZ() >> 4; cz++) {
                level.setChunkForced(cx, cz, true);
            }
        }
        // Entity loading is asynchronous, and LevelTicks will not run a chunk's
        // scheduled ticks until its entities are loaded. Wait for it rather than
        // guessing a tick count — a region that is loaded but not entity-ticking
        // looks exactly like redstone that ignores its input.
        int warmup = 0;
        while (!level.isPositionTickingWithEntitiesLoaded(centre.pack()) && warmup < 600) {
            tickServer.invoke(server, (BooleanSupplier) () -> true);
            waitUntilNextTick.invoke(server);
            warmup++;
        }
        System.out.printf("  warmup ticks until entity-ticking: %d (ready=%s)%n",
                warmup, level.isPositionTickingWithEntitiesLoaded(centre.pack()));

        for (BlockPos pos : BlockPos.betweenClosed(min, max)) {
            level.setBlock(pos, Blocks.AIR.defaultBlockState(), 2);
        }

        if (!template.placeInWorld(level, ORIGIN, ORIGIN, new StructurePlaceSettings(),
                RandomSource.create(0), 3)) {
            throw new IllegalStateException("failed to place " + structureId);
        }

        // Redstone settles synchronously while the structure is placed, so a trace
        // taken from here alone records nothing. To observe propagation we have to
        // disturb the settled state: --break removes a block (typically the power
        // source) and the ticks that follow are the circuit's response.
        // Decisive diagnostic: a region that is loaded but not in block-ticking
        // range looks exactly like a circuit that ignores its input.
        System.out.printf("  block-ticking here: %s%n",
                level.shouldTickBlocksAt(centre.pack()));
        System.out.printf("  pending block ticks: %d%n", level.getBlockTicks().count());
        System.out.printf("  runs normally: %s%n",
                level.tickRateManager().runsNormally());
        // LevelTicks gates on this, not on block-ticking range: a chunk can be
        // block-tickable yet have no active tick container if its entities are
        // not loaded, and then scheduled ticks sit pending forever.
        System.out.printf("  entities loaded / pos ticking: %s / %s%n",
                level.areEntitiesLoaded(centre.pack()),
                level.isPositionTickingWithEntitiesLoaded(centre.pack()));

        List<String> ticks = new ArrayList<>();
        Map<BlockPos, String> previous = snapshot(level, min, max);

        // --pulse places a power source at a position, holds it for --pulse-ticks,
        // then removes it. Short pulses are how several piston behaviours are
        // provoked, and they cannot be produced by --break alone.
        String pulseAt = arg(args, "--pulse", null);
        int pulseTicks = Integer.parseInt(arg(args, "--pulse-ticks", "1"));
        BlockPos pulsePos = null;
        if (pulseAt != null) {
            String[] pp = pulseAt.split(",");
            pulsePos = ORIGIN.offset(Integer.parseInt(pp[0].trim()),
                    Integer.parseInt(pp[1].trim()), Integer.parseInt(pp[2].trim()));
            level.setBlock(pulsePos, Blocks.REDSTONE_BLOCK.defaultBlockState(), 3);
            System.out.printf("  pulse: powered %s for %d tick(s)%n", pulseAt, pulseTicks);
        }
        final BlockPos pulseTarget = pulsePos;

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

        for (int tick = 0; tick < maxTicks; tick++) {
            if (pulseTarget != null && tick == pulseTicks) {
                level.setBlock(pulseTarget, Blocks.AIR.defaultBlockState(), 3);
            }
            tickServer.invoke(server, (BooleanSupplier) () -> true);
            waitUntilNextTick.invoke(server);
            if (tick < 3) {
                System.out.printf("    t%d gameTime=%d pending=%d%n",
                        tick, level.getGameTime(), level.getBlockTicks().count());
            }

            Map<BlockPos, String> current = snapshot(level, min, max);
            List<String> events = new ArrayList<>();
            for (BlockPos pos : BlockPos.betweenClosed(min, max)) {
                String was = previous.get(pos);
                String now = current.get(pos);
                if (!java.util.Objects.equals(was, now)) {
                    events.add(blockChanged(pos, was, now));
                }
            }
            previous = current;

            if (!events.isEmpty()) {
                ticks.add("    {\n      \"tick\": " + tick + ",\n      \"events\": [\n"
                        + String.join(",\n", events) + "\n      ]\n    }");
            }
        }

        String json = "{\n"
                + "  \"format_version\": 1,\n"
                + "  \"mc_version\": \"" + SharedConstants.getCurrentVersion().name() + "\",\n"
                + "  \"structure\": \"" + structureId + "\",\n"
                + "  \"detail\": \"normal\",\n"
                + "  \"ticks\": [\n" + String.join(",\n", ticks) + "\n  ]\n}\n";

        Path parent = out.getParent();
        if (parent != null) {
            Files.createDirectories(parent);
        }
        Files.writeString(out, json);
        System.out.printf("captured %d tick(s) with changes -> %s%n", ticks.size(), out);

        server.halt(false);
        System.exit(0);
    }

    /** Descriptors for every position in the box, so comparison is order-independent. */
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

    private static String arg(String[] args, String name, String fallback) {
        for (int i = 0; i < args.length - 1; i++) {
            if (args[i].equals(name)) {
                return args[i + 1];
            }
        }
        return fallback;
    }
}
