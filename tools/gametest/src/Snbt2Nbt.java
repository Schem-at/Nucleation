import net.minecraft.SharedConstants;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.NbtIo;
import net.minecraft.nbt.TagParser;
import net.minecraft.server.Bootstrap;

import java.nio.file.Files;
import java.nio.file.Path;

/**
 * Converts a structure {@code .snbt} to the binary {@code .nbt} that datapacks require.
 *
 * <p>Structures are authored as SNBT in this repo because text is reviewable and
 * diffable, and because it is the format the wider community's gametest suites use.
 * Datapacks, however, only load binary structures — vanilla ships zero {@code .snbt}
 * files in its own jar. SNBT loading does exist, but only through
 * {@code DirectoryTemplateSource(loadAsText = true)}, which reads from world
 * directories rather than datapacks.
 *
 * <p>The conversion deliberately uses the game's own {@code TagParser} and
 * {@code NbtIo} rather than a hand-rolled writer, so it cannot disagree with the
 * reader that will consume the result. A subtly wrong structure would fail as
 * "couldn't place the structure", which says nothing about the actual cause.
 */
public final class Snbt2Nbt {
    private Snbt2Nbt() {}

    public static void main(String[] args) throws Exception {
        if (args.length != 2) {
            System.err.println("usage: Snbt2Nbt <in.snbt> <out.nbt>");
            System.exit(2);
        }

        // Bootstrap.bootStrap() reads the version, and DataFixers dies at class-init
        // with "Game version not set" if this has not run first.
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();

        Path in = Path.of(args[0]);
        Path out = Path.of(args[1]);

        CompoundTag tag = TagParser.parseCompoundFully(Files.readString(in));
        Path parent = out.getParent();
        if (parent != null) {
            Files.createDirectories(parent);
        }
        NbtIo.writeCompressed(tag, out);

        System.out.printf("converted %s -> %s (%d bytes)%n", in, out, Files.size(out));
    }
}
