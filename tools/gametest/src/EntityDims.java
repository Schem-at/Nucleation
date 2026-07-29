import net.minecraft.SharedConstants;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.resources.Identifier;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.entity.EntityType;

/**
 * Print an entity type's hitbox, straight from the game's own registry.
 *
 * The record piston doors use entity hitboxes as mechanism — a dragon fireball
 * is chosen over a small one because it is tall enough to span a pressure plate
 * at the bottom of a block and the piston above it. Those dimensions therefore
 * have to be exact, and the only source that cannot disagree with the game is
 * the game. `EntityType.getWidth()/getHeight()` return the `sized(w, h)` the
 * type was registered with, and `getDimensions()` carries the eye height and
 * the "fixed" flag too.
 *
 *     java EntityDims minecraft:minecart minecraft:dragon_fireball ...
 */
public final class EntityDims {
    public static void main(String[] args) {
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();

        String[] wanted = args.length > 0
                ? args
                : new String[] {
                    "minecraft:minecart",
                    "minecraft:furnace_minecart",
                    "minecraft:chest_minecart",
                    "minecraft:hopper_minecart",
                    "minecraft:tnt_minecart",
                    "minecraft:dragon_fireball",
                    "minecraft:small_fireball",
                    "minecraft:fireball",
                    "minecraft:villager",
                    "minecraft:item",
                };

        System.out.printf("%-34s %-10s %-10s %-10s %s%n",
                "type", "width", "height", "eye", "fixed");
        for (String name : wanted) {
            Identifier id = Identifier.parse(name);
            EntityType<?> type = BuiltInRegistries.ENTITY_TYPE.getValue(id);
            if (type == null) {
                System.out.printf("%-34s UNKNOWN%n", name);
                continue;
            }
            var dims = type.getDimensions();
            System.out.printf("%-34s %-10s %-10s %-10s %s%n",
                    name,
                    fmt(dims.width()),
                    fmt(dims.height()),
                    fmt(dims.eyeHeight()),
                    dims.fixed());
        }
    }

    /** Exact float text — these feed AABB arithmetic, so no rounding. */
    private static String fmt(float value) {
        return Float.toString(value);
    }
}
