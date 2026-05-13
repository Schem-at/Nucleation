package examples;

import com.github.schemat.nucleation.*;
import java.nio.file.Files;
import java.nio.file.Path;

public final class NucleationDemo {

    public static void main(String[] args) throws Exception {
        System.out.println("Nucleation v" + Nucleation.version());
        System.out.println("Simulation: " + Nucleation.hasSimulation());

        if (args.length == 0) {
            // Build a 5x5x5 stone cube from scratch.
            try (Schematic s = new Schematic("demo-cube")) {
                s.fillCuboid(0, 0, 0, 4, 4, 4, "minecraft:stone");
                System.out.println("Built " + s.blockCount() + "-block schematic");
                byte[] bytes = s.toLitematic();
                Path out = Path.of("demo-cube.litematic");
                Files.write(out, bytes);
                System.out.println("Wrote " + out.toAbsolutePath() + " (" + bytes.length + " bytes)");
            }
            return;
        }

        // Otherwise: load the file passed as argv[0].
        byte[] data = Files.readAllBytes(Path.of(args[0]));
        try (Schematic s = Schematic.fromBytes(data)) {
            System.out.println("Loaded: " + s.name());
            System.out.println("Dimensions: " + s.dimensions());
            System.out.println("Block count: " + s.blockCount());
            System.out.println("Regions: " + s.regionNames());
            System.out.println("Top 5 block types:");
            s.countBlockTypes().entrySet().stream()
                .sorted((a, b) -> Integer.compare(b.getValue(), a.getValue()))
                .limit(5)
                .forEach(e -> System.out.println("  " + e.getKey() + " × " + e.getValue()));
        }
    }
}
