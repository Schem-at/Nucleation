import net.minecraft.SharedConstants;
import net.minecraft.core.BlockPos;
import net.minecraft.gametest.framework.GameTestServer;
import net.minecraft.server.Bootstrap;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.packs.repository.PackRepository;
import net.minecraft.server.packs.repository.ServerPacksSource;
import net.minecraft.util.Util;
import net.minecraft.world.level.block.state.BlockState;
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

/**
 * Captures a tick-by-tick trace of what the real game does, as ground truth for the
 * Rust engine.
 *
 * <h2>STATUS: INCOMPLETE — does not yet capture anything</h2>
 *
 * What works: the server is constructed by the same recipe {@code GameTestMainUtil}
 * uses, {@code initServer} and {@code tickServer} are driven manually one tick at a
 * time, and the test genuinely runs under that manual loop ("1 tests are now running
 * at position ..."). That was the hard part and it is done.
 *
 * <p>What does not: {@link #snapshot} returns nothing, so every run reports
 * "captured 0 tick(s) with changes". The reflected {@code visibleChunkMap} yields no
 * chunk covering the test region — either it is populated later than assumed, the
 * chunks are held somewhere else, or the accessor used is wrong ({@code
 * getTickingChunk} and {@code getChunkToSend} were both tried).
 *
 * <p>Do not treat output from this as ground truth until that is resolved; an empty
 * trace would diff clean against an engine that does nothing at all, which is exactly
 * the false-confidence failure this project must avoid. The likely next step is to
 * stop guessing at chunk iteration and instead scan a bounded box around the test's
 * reported origin, which the framework logs and can be obtained from the running
 * test rather than from the chunk map.
 *
 * <p>Correctness here is established by differential testing against vanilla, not by
 * reading its source. This is the half that produces the vanilla side.
 *
 * <h2>How it drives the game</h2>
 *
 * {@code GameTestMainUtil} hands control to {@link MinecraftServer#spin}, which runs
 * the server's own loop to completion — fine for pass/fail, useless for tracing,
 * because there is no point between ticks at which to observe. So this builds the
 * same {@link GameTestServer} by the same recipe and then drives {@code tickServer}
 * itself, one tick at a time, diffing the world in between.
 *
 * <p>{@code initServer} and {@code tickServer} are {@code protected}, so they are
 * called reflectively. That is ugly but honest: the alternative is bytecode
 * injection, which is far more machinery for the same access. Everything else is
 * public API, and the jar is unobfuscated, so these names are stable and readable.
 *
 * <h2>What it records, and what it does not</h2>
 *
 * Block changes, per tick, in scan order. It does <em>not</em> yet attribute an event
 * to the phase it happened in, because that needs interception inside the tick rather
 * than observation between ticks. Events are therefore emitted under the
 * {@code tick_end} sentinel phase, and the trace format's per-phase ordering is left
 * for a later pass rather than being fabricated here — a trace that guessed at phases
 * would be worse than one that admits it does not know.
 */
public final class TraceCapture {
    private TraceCapture() {}

    /** Sentinel used while we observe between ticks rather than within them. */
    private static final String PHASE_TICK_END = "tick_end";

    public static void main(String[] args) throws Exception {
        String universe = arg(args, "--universe", "work/trace-universe");
        String testFilter = arg(args, "--tests", "nucleation");
        int maxTicks = Integer.parseInt(arg(args, "--max-ticks", "200"));
        Path out = Path.of(arg(args, "--out", "work/trace.json"));

        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        Util.startTimerHackThread();

        Path universePath = Paths.get(universe);
        LevelStorageSource storage = LevelStorageSource.createDefault(universePath);
        LevelStorageSource.LevelStorageAccess access = storage.createAccess("gametestworld");
        PackRepository packs = ServerPacksSource.createPackRepository(access);

        GameTestServer server = GameTestServer.create(
                Thread.currentThread(), access, packs, Optional.of(testFilter), false, 1);

        Method initServer = MinecraftServer.class.getDeclaredMethod("initServer");
        Method tickServer = MinecraftServer.class.getDeclaredMethod(
                "tickServer", java.util.function.BooleanSupplier.class);
        initServer.setAccessible(true);
        tickServer.setAccessible(true);

        if (!(boolean) initServer.invoke(server)) {
            throw new IllegalStateException("initServer failed");
        }

        ServerLevel level = server.overworld();
        List<String> ticks = new ArrayList<>();
        Map<BlockPos, BlockState> previous = new HashMap<>();
        boolean seeded = false;

        for (int tick = 0; tick < maxTicks; tick++) {
            tickServer.invoke(server, (java.util.function.BooleanSupplier) () -> true);

            Map<BlockPos, BlockState> current = snapshot(level);
            if (!seeded) {
                // The first observation is the structure appearing, not a change
                // the simulation caused. Recording it would put a hundred phantom
                // events on tick 0.
                previous = current;
                seeded = true;
                continue;
            }

            List<String> events = new ArrayList<>();
            for (Map.Entry<BlockPos, BlockState> entry : current.entrySet()) {
                BlockState was = previous.get(entry.getKey());
                if (was == null || !was.equals(entry.getValue())) {
                    events.add(blockChanged(entry.getKey(), was, entry.getValue()));
                }
            }
            for (Map.Entry<BlockPos, BlockState> entry : previous.entrySet()) {
                if (!current.containsKey(entry.getKey())) {
                    events.add(blockChanged(entry.getKey(), entry.getValue(), null));
                }
            }
            previous = current;

            if (!events.isEmpty()) {
                ticks.add(String.format(
                        "    {%n      \"tick\": %d,%n      \"events\": [%n%s%n      ]%n    }",
                        tick, String.join(",%n".formatted(), events)));
            }
        }

        String json = String.format(
                "{%n  \"format_version\": 1,%n  \"mc_version\": \"%s\",%n"
                        + "  \"structure\": \"%s\",%n  \"detail\": \"normal\",%n"
                        + "  \"ticks\": [%n%s%n  ]%n}%n",
                SharedConstants.getCurrentVersion().name(),
                testFilter,
                String.join(",%n".formatted(), ticks));

        Path parent = out.getParent();
        if (parent != null) {
            Files.createDirectories(parent);
        }
        Files.writeString(out, json);
        System.out.printf("captured %d tick(s) with changes -> %s%n", ticks.size(), out);

        server.halt(false);
    }

    /**
     * Every non-air block in the loaded region.
     *
     * <p>Scanned rather than hooked, which is why phase attribution is absent. The
     * region is bounded by what the test framework loads, so this stays small.
     */
    @SuppressWarnings("unchecked")
    private static Map<BlockPos, BlockState> snapshot(ServerLevel level) throws Exception {
        Map<BlockPos, BlockState> blocks = new HashMap<>();

        // ChunkMap exposes no public iteration over loaded chunks, so read the
        // visible map reflectively — the same pragmatism as tickServer above, and
        // cheaper than the alternative of instrumenting the class.
        var chunkMap = level.getChunkSource().chunkMap;
        var field = chunkMap.getClass().getDeclaredField("visibleChunkMap");
        field.setAccessible(true);
        var visible = (it.unimi.dsi.fastutil.longs.Long2ObjectLinkedOpenHashMap<
                net.minecraft.server.level.ChunkHolder>) field.get(chunkMap);

        for (var holder : visible.values()) {
            // getTickingChunk() is null until a chunk reaches ticking status, which
            // it has not while the structure is still being placed. getChunkToSend()
            // is populated earlier, so it observes the region from the first tick.
            var chunk = holder.getChunkToSend();
            if (chunk == null) {
                continue;
            }
            var chunkPos = chunk.getPos();
            for (int y = level.getMinY(); y < level.getMaxY(); y++) {
                for (int x = 0; x < 16; x++) {
                    for (int z = 0; z < 16; z++) {
                        BlockPos pos = new BlockPos(chunkPos.getMinBlockX() + x, y,
                                chunkPos.getMinBlockZ() + z);
                        BlockState state = chunk.getBlockState(pos);
                        if (!state.isAir()) {
                            blocks.put(pos, state);
                        }
                    }
                }
            }
        }
        return blocks;
    }

    private static String blockChanged(BlockPos pos, BlockState from, BlockState to) {
        return String.format(
                "        {\"phase\": \"%s\", \"kind\": \"block_changed\", "
                        + "\"pos\": [%d, %d, %d], \"from\": \"%s\", \"to\": \"%s\"}",
                PHASE_TICK_END, pos.getX(), pos.getY(), pos.getZ(),
                describe(from), describe(to));
    }

    /** Descriptor string, matching the Rust side's state descriptors. */
    private static String describe(BlockState state) {
        if (state == null) {
            return "minecraft:air";
        }
        String text = state.toString();
        // BlockState.toString() renders as Block{minecraft:stone}[prop=value]; strip
        // the wrapper so the descriptor matches what the Rust registry interns.
        int open = text.indexOf('{');
        int close = text.indexOf('}');
        if (open >= 0 && close > open) {
            String name = text.substring(open + 1, close);
            String props = text.substring(close + 1);
            return name + props;
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
