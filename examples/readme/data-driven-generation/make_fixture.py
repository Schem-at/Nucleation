"""Create the tiny RGB image used by the data-driven generation guide."""

from pathlib import Path

from PIL import Image


WIDTH = 16
HEIGHT = 10


def rgb4(red, green, blue):
    """Expand exact four-bit channel values to eight-bit RGB."""
    return red * 17, green * 17, blue * 17


image = Image.new("RGB", (WIDTH, HEIGHT))
pixels = image.load()

face_left = 3

for y in range(HEIGHT):
    for x in range(WIDTH):
        # A night-sky border keeps the ten-wide creeper face readable while
        # giving all three channels some low-valued background data.
        sky = 1 + ((x * 3 + y * 5) % 3)
        pixels[x, y] = rgb4(1, sky, 3 + (sky & 1))

        if face_left <= x < face_left + 10:
            local_x = x - face_left
            mottled = (local_x * 5 + y * 3) % 4
            pixels[x, y] = rgb4(3 + mottled // 2, 9 + mottled, 3 + (mottled + y) % 3)

            eye = y in (2, 3) and local_x in (1, 2, 7, 8)
            nose = y in (4, 5) and local_x in (4, 5)
            jaw = y in (6, 7, 8) and local_x in (3, 4, 5, 6)
            notch = y == 9 and local_x in (3, 6)
            if eye or nose or jaw or notch:
                pixels[x, y] = rgb4(1, 2, 1)

root = Path(__file__).resolve().parents[3]
example = Path(__file__).with_name("rom-input.png")
published = root / "docs/media/readme/data-driven-generation/rom-input.png"
published.parent.mkdir(parents=True, exist_ok=True)

image.save(example, optimize=True)
image.save(published, optimize=True)
print(f"wrote {example} and {published}")
