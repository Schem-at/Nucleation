import net.minecraft.SharedConstants;
import net.minecraft.gametest.framework.GameTestMainUtil;

/**
 * Runs Minecraft's own {@code GameTestServer} headless over our test datapack.
 *
 * <p>This is the oracle: correctness for the Rust tick engine is established by
 * differential testing against the real game, not by reading its source. Running the
 * genuine article is therefore not a convenience but the whole point.
 *
 * <p>No mod loader is involved. Minecraft 26.2 ships its server jar unobfuscated, so
 * these classes are callable directly from a plain {@code javac} build against the
 * jar's own bundled classpath. And because {@code GameTestServer} runs in-process for
 * testing rather than serving players, no EULA acceptance is required.
 *
 * <p>Arguments are passed straight through to {@link GameTestMainUtil}:
 * {@code --universe <dir>}, {@code --packs <dir>}, {@code --report <file>},
 * {@code --tests <selector>}, {@code --verify <bool>}.
 */
public final class RunGameTests {
    private RunGameTests() {}

    public static void main(String[] args) throws Exception {
        // runGameTestServer calls Bootstrap.bootStrap() itself but *not* this, and
        // without it DataFixers fails at class-init with "Game version not set".
        SharedConstants.tryDetectVersion();

        GameTestMainUtil.runGameTestServer(args, thread -> {});
    }
}
