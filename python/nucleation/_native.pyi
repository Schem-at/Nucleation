"""
Type stubs for nucleation - A high-performance Minecraft schematic parser.

This file provides type hints for IDE autocomplete and type checking.
"""

from typing import Optional, Dict, List, Tuple, Any


class BlockState:
    """Represents a Minecraft block state with name and properties."""

    def __init__(self, name: str) -> None:
        """Create a new BlockState with the given name."""
        ...

    def with_property(self, key: str, value: str) -> "BlockState":
        """Return a new BlockState with an additional property."""
        ...

    @property
    def name(self) -> str:
        """Get the block name (e.g., 'minecraft:stone')."""
        ...

    @property
    def properties(self) -> Dict[str, str]:
        """Get the block properties as a dictionary."""
        ...

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...


class Schematic:
    """
    Represents a Minecraft schematic with blocks, metadata, and regions.

    Supports loading/saving .litematic and .schematic formats.
    """

    def __init__(self, name: Optional[str] = None) -> None:
        """Create a new empty schematic."""
        ...

    def test(self) -> str:
        """Test method to verify the class is working."""
        ...

    def from_data(self, data: bytes) -> None:
        """Load schematic from raw bytes. Auto-detects format."""
        ...

    def from_litematic(self, data: bytes) -> None:
        """Load schematic from Litematica format bytes."""
        ...

    def to_litematic(self) -> bytes:
        """Export schematic to Litematica format."""
        ...

    def from_schematic(self, data: bytes) -> None:
        """Load schematic from WorldEdit/Sponge format bytes."""
        ...

    def to_schematic(self) -> bytes:
        """Export schematic to WorldEdit/Sponge format."""
        ...

    def from_mcstructure(self, data: bytes) -> None:
        """Load schematic from McStructure (Bedrock) format bytes."""
        ...

    def to_mcstructure(self) -> bytes:
        """Export schematic to McStructure (Bedrock) format."""
        ...

    # --- Metadata Accessors ---

    @property
    def name(self) -> Optional[str]:
        """Get the schematic name."""
        ...

    @name.setter
    def name(self, value: str) -> None:
        """Set the schematic name."""
        ...

    @property
    def author(self) -> Optional[str]:
        """Get the schematic author."""
        ...

    @author.setter
    def author(self, value: str) -> None:
        """Set the schematic author."""
        ...

    @property
    def description(self) -> Optional[str]:
        """Get the schematic description."""
        ...

    @description.setter
    def description(self, value: str) -> None:
        """Set the schematic description."""
        ...

    @property
    def created(self) -> Optional[int]:
        """Get the creation timestamp (milliseconds since epoch)."""
        ...

    @created.setter
    def created(self, value: int) -> None:
        """Set the creation timestamp."""
        ...

    @property
    def modified(self) -> Optional[int]:
        """Get the modification timestamp (milliseconds since epoch)."""
        ...

    @modified.setter
    def modified(self, value: int) -> None:
        """Set the modification timestamp."""
        ...

    @property
    def lm_version(self) -> Optional[int]:
        """Get the Litematic format version."""
        ...

    @lm_version.setter
    def lm_version(self, value: int) -> None:
        """Set the Litematic format version."""
        ...

    @property
    def mc_version(self) -> Optional[int]:
        """Get the Minecraft data version."""
        ...

    @mc_version.setter
    def mc_version(self, value: int) -> None:
        """Set the Minecraft data version."""
        ...

    @property
    def we_version(self) -> Optional[int]:
        """Get the WorldEdit version."""
        ...

    @we_version.setter
    def we_version(self, value: int) -> None:
        """Set the WorldEdit version."""
        ...

    # --- Block Operations ---

    def set_block(self, x: int, y: int, z: int, block_name: str) -> bool:
        """Set a block at the given position using block string notation."""
        ...

    def set_block_in_region(
        self, region_name: str, x: int, y: int, z: int, block_name: str
    ) -> bool:
        """Set a block in a specific region."""
        ...

    def clear_cache(self) -> None:
        """Clear the internal block state cache."""
        ...

    def cache_info(self) -> Tuple[int, int]:
        """Get cache statistics as (cache_size, cache_hits)."""
        ...

    def set_block_from_string(
        self, x: int, y: int, z: int, block_string: str
    ) -> None:
        """Set a block using full string notation."""
        ...

    def set_block_with_properties(
        self,
        x: int,
        y: int,
        z: int,
        block_name: str,
        properties: Dict[str, str],
    ) -> None:
        """Set a block with explicit properties dictionary."""
        ...

    def set_block_with_nbt(
        self,
        x: int,
        y: int,
        z: int,
        block_name: str,
        nbt_data: Dict[str, str],
    ) -> None:
        """Set a block with NBT data."""
        ...

    def copy_region(
        self,
        from_schematic: "Schematic",
        min_x: int,
        min_y: int,
        min_z: int,
        max_x: int,
        max_y: int,
        max_z: int,
        target_x: int,
        target_y: int,
        target_z: int,
        excluded_blocks: Optional[List[str]] = None,
    ) -> None:
        """Copy a region from another schematic."""
        ...

    def get_block(self, x: int, y: int, z: int) -> Optional[BlockState]:
        """Get the block state at the given position."""
        ...

    def get_block_string(self, x: int, y: int, z: int) -> Optional[str]:
        """Get block as formatted string with properties."""
        ...

    def get_block_with_properties(
        self, x: int, y: int, z: int
    ) -> Optional[BlockState]:
        """Get block with full properties at position."""
        ...

    def get_palette(self) -> List[BlockState]:
        """Get the palette (unique block types) for the default region."""
        ...

    def get_block_entity(
        self, x: int, y: int, z: int
    ) -> Optional[Dict[str, Any]]:
        """Get block entity (tile entity) data at position."""
        ...

    def get_all_block_entities(self) -> List[Dict[str, Any]]:
        """Get all block entities in the schematic."""
        ...

    def get_all_blocks(self) -> List[Dict[str, Any]]:
        """Get all blocks with their positions and properties."""
        ...

    def get_chunks(
        self,
        chunk_width: int,
        chunk_height: int,
        chunk_length: int,
        strategy: Optional[str] = None,
        camera_x: float = 0.0,
        camera_y: float = 0.0,
        camera_z: float = 0.0,
    ) -> List[Dict[str, Any]]:
        """Get schematic data organized into chunks."""
        ...

    def fill_cuboid(
        self,
        min_x: int,
        min_y: int,
        min_z: int,
        max_x: int,
        max_y: int,
        max_z: int,
        block_state: str,
    ) -> None:
        """Fill a cuboid region with a block."""
        ...

    def fill_sphere(
        self, cx: int, cy: int, cz: int, radius: float, block_state: str
    ) -> None:
        """Fill a sphere with a block."""
        ...

    def flip_x(self) -> None:
        """Flip the schematic along the X axis."""
        ...

    def flip_y(self) -> None:
        """Flip the schematic along the Y axis."""
        ...

    def flip_z(self) -> None:
        """Flip the schematic along the Z axis."""
        ...

    def rotate_x(self, degrees: int) -> None:
        """Rotate the schematic around the X axis."""
        ...

    def rotate_y(self, degrees: int) -> None:
        """Rotate the schematic around the Y axis."""
        ...

    def rotate_z(self, degrees: int) -> None:
        """Rotate the schematic around the Z axis."""
        ...

    def flip_region_x(self, region_name: str) -> None:
        """Flip a specific region along the X axis."""
        ...

    def flip_region_y(self, region_name: str) -> None:
        """Flip a specific region along the Y axis."""
        ...

    def flip_region_z(self, region_name: str) -> None:
        """Flip a specific region along the Z axis."""
        ...

    def rotate_region_x(self, region_name: str, degrees: int) -> None:
        """Rotate a specific region around the X axis."""
        ...

    def rotate_region_y(self, region_name: str, degrees: int) -> None:
        """Rotate a specific region around the Y axis."""
        ...

    def rotate_region_z(self, region_name: str, degrees: int) -> None:
        """Rotate a specific region around the Z axis."""
        ...

    def extract_signs(self) -> List[Dict[str, Any]]:
        """Extract sign data from the schematic."""
        ...

    def compile_insign(self) -> Dict[str, Any]:
        """Compile InSign circuit definitions from sign data."""
        ...

    def debug_info(self) -> str:
        """Get debug information about the schematic."""
        ...

    def print_schematic(self) -> str:
        """Get a printable string representation of the schematic."""
        ...

    # --- Format management ---

    def save_as(self, format: str, version: Optional[str] = None) -> bytes:
        """Export schematic to a specific format and version."""
        ...

    def to_schematic_version(self, version: str) -> bytes:
        """Export schematic to a specific Sponge schematic version."""
        ...

    @staticmethod
    def get_supported_import_formats() -> List[str]:
        """Get list of supported import formats."""
        ...

    @staticmethod
    def get_supported_export_formats() -> List[str]:
        """Get list of supported export formats."""
        ...

    @staticmethod
    def get_format_versions(format: str) -> List[str]:
        """Get available versions for a format."""
        ...

    @staticmethod
    def get_default_format_version(format: str) -> Optional[str]:
        """Get the default version for a format."""
        ...

    @staticmethod
    def get_available_schematic_versions() -> List[str]:
        """Get available Sponge schematic versions."""
        ...

    # --- Palette methods ---

    def get_all_palettes(self) -> Dict[str, List[str]]:
        """Get all palettes from all regions."""
        ...

    def get_default_region_palette(self) -> List[str]:
        """Get the palette from the default region."""
        ...

    def get_palette_from_region(self, region_name: str) -> List[str]:
        """Get the palette from a specific region."""
        ...

    # --- Bounding box / dimensions ---

    def get_bounding_box(self) -> Tuple[Tuple[int, int, int], Tuple[int, int, int]]:
        """Get the bounding box as ((min_x, min_y, min_z), (max_x, max_y, max_z))."""
        ...

    def get_region_bounding_box(
        self, region_name: str
    ) -> Tuple[Tuple[int, int, int], Tuple[int, int, int]]:
        """Get the bounding box of a specific region."""
        ...

    def get_tight_dimensions(self) -> Tuple[int, int, int]:
        """Get tight dimensions (excluding air padding)."""
        ...

    def get_tight_bounds_min(self) -> Optional[Tuple[int, int, int]]:
        """Get the minimum corner of the tight bounding box."""
        ...

    def get_tight_bounds_max(self) -> Optional[Tuple[int, int, int]]:
        """Get the maximum corner of the tight bounding box."""
        ...

    # --- Definition region management ---

    def add_definition_region(
        self, name: str, region: "DefinitionRegion"
    ) -> None:
        """Add a definition region to the schematic."""
        ...

    def get_definition_region(self, name: str) -> "DefinitionRegion":
        """Get a definition region by name."""
        ...

    def create_region(
        self,
        name: str,
        min: Tuple[int, int, int],
        max: Tuple[int, int, int],
    ) -> "DefinitionRegion":
        """Create a new region with bounds."""
        ...

    def remove_definition_region(self, name: str) -> bool:
        """Remove a definition region by name."""
        ...

    def get_definition_region_names(self) -> List[str]:
        """Get all definition region names."""
        ...

    def create_definition_region(self, name: str) -> None:
        """Create an empty definition region."""
        ...

    def create_definition_region_from_point(
        self, name: str, x: int, y: int, z: int
    ) -> None:
        """Create a definition region from a single point."""
        ...

    def create_definition_region_from_bounds(
        self,
        name: str,
        min: Tuple[int, int, int],
        max: Tuple[int, int, int],
    ) -> None:
        """Create a definition region from bounding box corners."""
        ...

    def definition_region_add_bounds(
        self,
        name: str,
        min: Tuple[int, int, int],
        max: Tuple[int, int, int],
    ) -> None:
        """Add bounding box to an existing definition region."""
        ...

    def definition_region_add_point(
        self, name: str, x: int, y: int, z: int
    ) -> None:
        """Add a point to an existing definition region."""
        ...

    def definition_region_set_metadata(
        self, name: str, key: str, value: str
    ) -> None:
        """Set metadata on a definition region."""
        ...

    def definition_region_shift(
        self, name: str, x: int, y: int, z: int
    ) -> None:
        """Shift a definition region by offset."""
        ...

    def update_region(self, name: str, region: "DefinitionRegion") -> None:
        """Update an existing definition region."""
        ...

    # --- Properties ---

    @property
    def dimensions(self) -> Tuple[int, int, int]:
        """Get schematic dimensions as (width, height, depth)."""
        ...

    @property
    def allocated_dimensions(self) -> Tuple[int, int, int]:
        """Get allocated dimensions."""
        ...

    @property
    def block_count(self) -> int:
        """Get total number of non-air blocks."""
        ...

    @property
    def volume(self) -> int:
        """Get total volume (width * height * depth)."""
        ...

    @property
    def region_names(self) -> List[str]:
        """Get list of region names in this schematic."""
        ...

    # --- Simulation (requires 'simulation' feature) ---

    def create_simulation_world(self) -> "MchprsWorld":
        """Create a redstone simulation world from this schematic."""
        ...

    def build_executor(
        self,
        inputs: List[Dict[str, str]],
        outputs: List[Dict[str, str]],
    ) -> "TypedCircuitExecutor":
        """Build a typed circuit executor from input/output definitions."""
        ...

    # --- Meshing (requires 'meshing' feature) ---

    def to_mesh(
        self,
        pack: "ResourcePack",
        config: Optional["MeshConfig"] = None,
    ) -> "MeshResult":
        """Generate a 3D mesh from the schematic."""
        ...

    def mesh_by_region(
        self,
        pack: "ResourcePack",
        config: Optional["MeshConfig"] = None,
    ) -> "MultiMeshResult":
        """Generate meshes per region."""
        ...

    def mesh_by_chunk(
        self,
        pack: "ResourcePack",
        config: Optional["MeshConfig"] = None,
    ) -> "ChunkMeshResult":
        """Generate meshes per chunk."""
        ...

    def mesh_by_chunk_size(
        self,
        pack: "ResourcePack",
        chunk_size: int,
        config: Optional["MeshConfig"] = None,
    ) -> "ChunkMeshResult":
        """Generate meshes per custom-sized chunk."""
        ...

    def to_usdz(
        self,
        pack: "ResourcePack",
        config: Optional["MeshConfig"] = None,
    ) -> "MeshResult":
        """Generate a USDZ mesh from the schematic."""
        ...

    def to_raw_mesh(
        self,
        pack: "ResourcePack",
        config: Optional["MeshConfig"] = None,
    ) -> "RawMeshExport":
        """Generate raw mesh data for custom rendering pipelines."""
        ...

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...


class DefinitionRegion:
    """Represents a spatial region defined by bounding boxes."""

    def __init__(self) -> None:
        """Create a new empty definition region."""
        ...

    @staticmethod
    def from_bounds(
        min: Tuple[int, int, int], max: Tuple[int, int, int]
    ) -> "DefinitionRegion":
        """Create a region from bounding box corners."""
        ...

    @staticmethod
    def from_bounding_boxes(
        boxes: List[Tuple[Tuple[int, int, int], Tuple[int, int, int]]]
    ) -> "DefinitionRegion":
        """Create a region from multiple bounding boxes."""
        ...

    @staticmethod
    def from_positions(
        positions: List[Tuple[int, int, int]]
    ) -> "DefinitionRegion":
        """Create a region from a list of positions."""
        ...

    def add_bounds(
        self, min: Tuple[int, int, int], max: Tuple[int, int, int]
    ) -> "DefinitionRegion":
        """Add a bounding box to this region. Returns self for chaining."""
        ...

    def add_point(self, x: int, y: int, z: int) -> "DefinitionRegion":
        """Add a single point to this region. Returns self for chaining."""
        ...

    def set_metadata(self, key: str, value: str) -> "DefinitionRegion":
        """Set metadata on this region. Returns self for chaining."""
        ...

    def set_metadata_mut(self, key: str, value: str) -> None:
        """Set metadata on this region (in-place)."""
        ...

    def get_metadata(self, key: str) -> Optional[str]:
        """Get metadata value by key."""
        ...

    def get_all_metadata(self) -> Dict[str, str]:
        """Get all metadata as a dictionary."""
        ...

    def metadata_keys(self) -> List[str]:
        """Get all metadata keys."""
        ...

    def set_color(self, color: int) -> "DefinitionRegion":
        """Set the display color. Returns self for chaining."""
        ...

    def add_filter(self, filter: str) -> "DefinitionRegion":
        """Add a block filter. Returns self for chaining."""
        ...

    def exclude_block(self, block_name: str) -> "DefinitionRegion":
        """Exclude a block type. Returns self for chaining."""
        ...

    def merge(self, other: "DefinitionRegion") -> "DefinitionRegion":
        """Merge another region into this one. Returns self for chaining."""
        ...

    def subtract(self, other: "DefinitionRegion") -> None:
        """Subtract another region from this one (in-place)."""
        ...

    def intersect(self, other: "DefinitionRegion") -> None:
        """Intersect with another region (in-place)."""
        ...

    def union_into(self, other: "DefinitionRegion") -> None:
        """Union another region into this one (in-place)."""
        ...

    def union(self, other: "DefinitionRegion") -> "DefinitionRegion":
        """Return a new region that is the union of both."""
        ...

    def subtracted(self, other: "DefinitionRegion") -> "DefinitionRegion":
        """Return a new region with other subtracted."""
        ...

    def intersected(self, other: "DefinitionRegion") -> "DefinitionRegion":
        """Return a new region that is the intersection of both."""
        ...

    def shift(self, x: int, y: int, z: int) -> None:
        """Shift the region by offset (in-place)."""
        ...

    def expand(self, x: int, y: int, z: int) -> None:
        """Expand the region (in-place)."""
        ...

    def contract(self, amount: int) -> None:
        """Contract the region (in-place)."""
        ...

    def shifted(self, x: int, y: int, z: int) -> "DefinitionRegion":
        """Return a new shifted region."""
        ...

    def expanded(self, x: int, y: int, z: int) -> "DefinitionRegion":
        """Return a new expanded region."""
        ...

    def contracted(self, amount: int) -> "DefinitionRegion":
        """Return a new contracted region."""
        ...

    def copy(self) -> "DefinitionRegion":
        """Create a copy of this region."""
        ...

    def get_bounds(
        self,
    ) -> Optional[Tuple[Tuple[int, int, int], Tuple[int, int, int]]]:
        """Get the overall bounding box, or None if empty."""
        ...

    def is_contiguous(self) -> bool:
        """Check if the region is contiguous."""
        ...

    def connected_components(self) -> int:
        """Count the number of connected components."""
        ...

    def filter_by_block(
        self, schematic: Schematic, block_name: str
    ) -> "DefinitionRegion":
        """Filter to positions containing a specific block."""
        ...

    def filter_by_properties(
        self, schematic: Schematic, properties: Dict[str, str]
    ) -> "DefinitionRegion":
        """Filter to positions matching block properties."""
        ...

    def is_empty(self) -> bool:
        """Check if the region is empty."""
        ...

    def contains(self, x: int, y: int, z: int) -> bool:
        """Check if a position is contained in this region."""
        ...

    def volume(self) -> int:
        """Get the total volume of the region."""
        ...

    def positions(self) -> List[Tuple[int, int, int]]:
        """Get all positions in this region."""
        ...

    def positions_sorted(self) -> List[Tuple[int, int, int]]:
        """Get all positions sorted."""
        ...

    def simplify(self) -> None:
        """Simplify the region by merging overlapping boxes."""
        ...

    def box_count(self) -> int:
        """Get the number of bounding boxes."""
        ...

    def get_box(
        self, index: int
    ) -> Optional[Tuple[Tuple[int, int, int], Tuple[int, int, int]]]:
        """Get a bounding box by index."""
        ...

    def get_boxes(
        self,
    ) -> List[Tuple[Tuple[int, int, int], Tuple[int, int, int]]]:
        """Get all bounding boxes."""
        ...

    def dimensions(self) -> Tuple[int, int, int]:
        """Get the dimensions of the bounding box."""
        ...

    def center(self) -> Optional[Tuple[int, int, int]]:
        """Get the integer center point."""
        ...

    def center_f32(self) -> Optional[Tuple[float, float, float]]:
        """Get the floating-point center."""
        ...

    def intersects_bounds(
        self, min: Tuple[int, int, int], max: Tuple[int, int, int]
    ) -> bool:
        """Check if this region intersects a bounding box."""
        ...

    def sync(self) -> None:
        """Sync region state."""
        ...

    def __repr__(self) -> str: ...
    def __len__(self) -> int: ...
    def __bool__(self) -> bool: ...
    def __copy__(self) -> "DefinitionRegion": ...
    def __deepcopy__(self, memo: Any) -> "DefinitionRegion": ...


class SchematicBuilder:
    """Builder for creating schematics from ASCII art or templates."""

    def __init__(self) -> None:
        """Create a new schematic builder."""
        ...

    def name(self, name: str) -> "SchematicBuilder":
        """Set the schematic name. Returns self for chaining."""
        ...

    def map(self, ch: str, block: str) -> "SchematicBuilder":
        """Map a character to a block type. Returns self for chaining."""
        ...

    def layers(self, layers: List[List[str]]) -> "SchematicBuilder":
        """Add layers of ASCII art. Returns self for chaining."""
        ...

    def build(self) -> Schematic:
        """Build the schematic."""
        ...

    @staticmethod
    def from_template(template: str) -> "SchematicBuilder":
        """Create a builder from a template string."""
        ...

    def __repr__(self) -> str: ...


class Shape:
    """Represents a geometric shape for building operations."""

    @staticmethod
    def sphere(cx: int, cy: int, cz: int, radius: float) -> "Shape":
        """Create a sphere shape."""
        ...

    @staticmethod
    def cuboid(
        min_x: int,
        min_y: int,
        min_z: int,
        max_x: int,
        max_y: int,
        max_z: int,
    ) -> "Shape":
        """Create a cuboid shape."""
        ...


class Brush:
    """Represents a brush for painting blocks."""

    @staticmethod
    def solid(block_state: str) -> "Brush":
        """Create a solid brush with a specific block."""
        ...

    @staticmethod
    def color(
        r: int,
        g: int,
        b: int,
        palette_filter: Optional[List[str]] = None,
    ) -> "Brush":
        """Create a color brush (matches closest block to RGB color)."""
        ...

    @staticmethod
    def linear_gradient(
        x1: int,
        y1: int,
        z1: int,
        r1: int,
        g1: int,
        b1: int,
        x2: int,
        y2: int,
        z2: int,
        r2: int,
        g2: int,
        b2: int,
        space: Optional[int] = None,
        palette_filter: Optional[List[str]] = None,
    ) -> "Brush":
        """Create a linear gradient brush. Space: 0=RGB, 1=Oklab."""
        ...

    @staticmethod
    def bilinear_gradient(
        ox: int,
        oy: int,
        oz: int,
        ux: int,
        uy: int,
        uz: int,
        vx: int,
        vy: int,
        vz: int,
        r00: int,
        g00: int,
        b00: int,
        r10: int,
        g10: int,
        b10: int,
        r01: int,
        g01: int,
        b01: int,
        r11: int,
        g11: int,
        b11: int,
        space: Optional[int] = None,
        palette_filter: Optional[List[str]] = None,
    ) -> "Brush":
        """Create a bilinear gradient brush (4-corner quad)."""
        ...

    @staticmethod
    def shaded(
        r: int,
        g: int,
        b: int,
        lx: float,
        ly: float,
        lz: float,
        palette_filter: Optional[List[str]] = None,
    ) -> "Brush":
        """Create a shaded brush (Lambertian shading)."""
        ...

    @staticmethod
    def point_gradient(
        points: List[Tuple[Tuple[int, int, int], Tuple[int, int, int]]],
        falloff: Optional[float] = None,
        space: Optional[int] = None,
        palette_filter: Optional[List[str]] = None,
    ) -> "Brush":
        """Create a point cloud gradient brush using IDW interpolation."""
        ...


class BuildingTool:
    """Tool for applying brushes to shapes on schematics."""

    @staticmethod
    def fill(schematic: Schematic, shape: Shape, brush: Brush) -> None:
        """Apply a brush to a shape on the given schematic."""
        ...


# ============================================================================
# Simulation types (requires 'simulation' feature)
# ============================================================================


class MchprsWorld:
    """Redstone circuit simulation world powered by MCHPRS."""

    def on_use_block(self, x: int, y: int, z: int) -> None:
        """Simulate right-clicking a block (e.g., toggling a lever)."""
        ...

    def tick(self, ticks: int) -> None:
        """Advance the simulation by a number of ticks."""
        ...

    def flush(self) -> None:
        """Flush pending simulation changes to the world."""
        ...

    def is_lit(self, x: int, y: int, z: int) -> bool:
        """Check if a redstone lamp is lit at the given position."""
        ...

    def get_lever_power(self, x: int, y: int, z: int) -> bool:
        """Get the power state of a lever."""
        ...

    def get_redstone_power(self, x: int, y: int, z: int) -> int:
        """Get redstone power level at position (0-15)."""
        ...

    def set_signal_strength(
        self, x: int, y: int, z: int, strength: int
    ) -> None:
        """Set signal strength at a position."""
        ...

    def get_signal_strength(self, x: int, y: int, z: int) -> int:
        """Get signal strength at a position."""
        ...

    def check_custom_io_changes(self) -> None:
        """Check for custom I/O changes."""
        ...

    def poll_custom_io_changes(self) -> Any:
        """Poll and consume custom I/O changes."""
        ...

    def peek_custom_io_changes(self) -> Any:
        """Peek at custom I/O changes without consuming."""
        ...

    def clear_custom_io_changes(self) -> None:
        """Clear pending custom I/O changes."""
        ...

    def sync_to_schematic(self) -> None:
        """Sync current simulation state back to the schematic."""
        ...

    def get_schematic(self) -> Schematic:
        """Get the underlying schematic."""
        ...

    def into_schematic(self) -> Schematic:
        """Consume the world and return the schematic with synced state."""
        ...

    def __repr__(self) -> str: ...


class Value:
    """A typed value for circuit I/O."""

    @staticmethod
    def u32(value: int) -> "Value":
        """Create an unsigned 32-bit integer value."""
        ...

    @staticmethod
    def i32(value: int) -> "Value":
        """Create a signed 32-bit integer value."""
        ...

    @staticmethod
    def f32(value: float) -> "Value":
        """Create a 32-bit float value."""
        ...

    @staticmethod
    def bool(value: bool) -> "Value":
        """Create a boolean value."""
        ...

    @staticmethod
    def string(value: str) -> "Value":
        """Create a string value."""
        ...

    def to_py(self) -> Any:
        """Convert to a native Python value."""
        ...

    def type_name(self) -> str:
        """Get the type name of this value."""
        ...

    def __repr__(self) -> str: ...


class IoType:
    """Defines the type of a circuit I/O port."""

    @staticmethod
    def unsigned_int(bits: int) -> "IoType":
        """Unsigned integer with specified bit width."""
        ...

    @staticmethod
    def signed_int(bits: int) -> "IoType":
        """Signed integer with specified bit width."""
        ...

    @staticmethod
    def float32() -> "IoType":
        """32-bit floating point."""
        ...

    @staticmethod
    def boolean() -> "IoType":
        """Boolean type."""
        ...

    @staticmethod
    def ascii(chars: int) -> "IoType":
        """ASCII string with specified character count."""
        ...

    def __repr__(self) -> str: ...


class LayoutFunction:
    """Maps circuit I/O positions to value bits."""

    @staticmethod
    def one_to_one() -> "LayoutFunction":
        """One-to-one mapping (position index = bit index)."""
        ...

    @staticmethod
    def packed4() -> "LayoutFunction":
        """Packed 4-bit mapping."""
        ...

    @staticmethod
    def custom(mapping: List[int]) -> "LayoutFunction":
        """Custom bit mapping."""
        ...

    @staticmethod
    def row_major(
        rows: int, cols: int, bits_per_element: int
    ) -> "LayoutFunction":
        """Row-major matrix layout."""
        ...

    @staticmethod
    def column_major(
        rows: int, cols: int, bits_per_element: int
    ) -> "LayoutFunction":
        """Column-major matrix layout."""
        ...

    @staticmethod
    def scanline(
        width: int, height: int, bits_per_pixel: int
    ) -> "LayoutFunction":
        """Scanline layout for 2D data."""
        ...

    def __repr__(self) -> str: ...


class OutputCondition:
    """Condition for checking circuit output values."""

    @staticmethod
    def equals(value: Value) -> "OutputCondition":
        """Match when output equals value."""
        ...

    @staticmethod
    def not_equals(value: Value) -> "OutputCondition":
        """Match when output does not equal value."""
        ...

    @staticmethod
    def greater_than(value: Value) -> "OutputCondition":
        """Match when output is greater than value."""
        ...

    @staticmethod
    def less_than(value: Value) -> "OutputCondition":
        """Match when output is less than value."""
        ...

    @staticmethod
    def bitwise_and(mask: int) -> "OutputCondition":
        """Match when output AND mask is non-zero."""
        ...

    def __repr__(self) -> str: ...


class ExecutionMode:
    """Controls how circuit simulation is executed."""

    @staticmethod
    def fixed_ticks(ticks: int) -> "ExecutionMode":
        """Run for a fixed number of ticks."""
        ...

    @staticmethod
    def until_condition(
        output_name: str,
        condition: OutputCondition,
        max_ticks: int,
        check_interval: int,
    ) -> "ExecutionMode":
        """Run until an output matches a condition."""
        ...

    @staticmethod
    def until_change(max_ticks: int, check_interval: int) -> "ExecutionMode":
        """Run until any output changes."""
        ...

    @staticmethod
    def until_stable(stable_ticks: int, max_ticks: int) -> "ExecutionMode":
        """Run until outputs are stable for N ticks."""
        ...

    def __repr__(self) -> str: ...


class SortStrategy:
    """Strategy for sorting I/O positions in circuits."""

    @staticmethod
    def yxz() -> "SortStrategy":
        """Sort by Y, then X, then Z."""
        ...

    @staticmethod
    def xyz() -> "SortStrategy":
        """Sort by X, then Y, then Z."""
        ...

    @staticmethod
    def zyx() -> "SortStrategy":
        """Sort by Z, then Y, then X."""
        ...

    @staticmethod
    def y_desc_xz() -> "SortStrategy":
        """Sort by Y descending, then X, then Z."""
        ...

    @staticmethod
    def x_desc_yz() -> "SortStrategy":
        """Sort by X descending, then Y, then Z."""
        ...

    @staticmethod
    def z_desc_yx() -> "SortStrategy":
        """Sort by Z descending, then Y, then X."""
        ...

    @staticmethod
    def descending() -> "SortStrategy":
        """Sort in descending order."""
        ...

    @staticmethod
    def distance_from(x: int, y: int, z: int) -> "SortStrategy":
        """Sort by distance from a point (ascending)."""
        ...

    @staticmethod
    def distance_from_desc(x: int, y: int, z: int) -> "SortStrategy":
        """Sort by distance from a point (descending)."""
        ...

    @staticmethod
    def preserve() -> "SortStrategy":
        """Preserve original order."""
        ...

    @staticmethod
    def reverse() -> "SortStrategy":
        """Reverse order."""
        ...

    @staticmethod
    def from_string(s: str) -> "SortStrategy":
        """Create from string name."""
        ...

    @property
    def name(self) -> str:
        """Get the strategy name."""
        ...

    def __repr__(self) -> str: ...


class IoLayout:
    """Describes the I/O layout for a circuit."""

    def input_names(self) -> List[str]:
        """Get input port names."""
        ...

    def output_names(self) -> List[str]:
        """Get output port names."""
        ...

    def __repr__(self) -> str: ...


class IoLayoutBuilder:
    """Builder for constructing I/O layouts."""

    def __init__(self) -> None:
        """Create a new IoLayoutBuilder."""
        ...

    def add_input(
        self,
        name: str,
        io_type: IoType,
        layout: LayoutFunction,
        positions: List[Tuple[int, int, int]],
    ) -> "IoLayoutBuilder":
        """Add an input port with explicit layout."""
        ...

    def add_output(
        self,
        name: str,
        io_type: IoType,
        layout: LayoutFunction,
        positions: List[Tuple[int, int, int]],
    ) -> "IoLayoutBuilder":
        """Add an output port with explicit layout."""
        ...

    def add_input_auto(
        self,
        name: str,
        io_type: IoType,
        positions: List[Tuple[int, int, int]],
    ) -> "IoLayoutBuilder":
        """Add an input port with automatic layout."""
        ...

    def add_output_auto(
        self,
        name: str,
        io_type: IoType,
        positions: List[Tuple[int, int, int]],
    ) -> "IoLayoutBuilder":
        """Add an output port with automatic layout."""
        ...

    def add_input_from_region(
        self,
        name: str,
        io_type: IoType,
        layout: LayoutFunction,
        region: DefinitionRegion,
    ) -> "IoLayoutBuilder":
        """Add an input port from a definition region."""
        ...

    def add_input_from_region_auto(
        self,
        name: str,
        io_type: IoType,
        region: DefinitionRegion,
    ) -> "IoLayoutBuilder":
        """Add an input port from a region with automatic layout."""
        ...

    def add_output_from_region(
        self,
        name: str,
        io_type: IoType,
        layout: LayoutFunction,
        region: DefinitionRegion,
    ) -> "IoLayoutBuilder":
        """Add an output port from a definition region."""
        ...

    def add_output_from_region_auto(
        self,
        name: str,
        io_type: IoType,
        region: DefinitionRegion,
    ) -> "IoLayoutBuilder":
        """Add an output port from a region with automatic layout."""
        ...

    def build(self) -> IoLayout:
        """Build the I/O layout."""
        ...

    def __repr__(self) -> str: ...


class CircuitBuilder:
    """Builder for constructing typed circuit executors."""

    def __init__(self, schematic: Schematic) -> None:
        """Create a new CircuitBuilder from a schematic."""
        ...

    @staticmethod
    def from_insign(schematic: Schematic) -> "CircuitBuilder":
        """Create from InSign definitions in the schematic."""
        ...

    def with_input(
        self,
        name: str,
        io_type: IoType,
        layout: LayoutFunction,
        region: DefinitionRegion,
    ) -> None:
        """Add an input port."""
        ...

    def with_input_sorted(
        self,
        name: str,
        io_type: IoType,
        layout: LayoutFunction,
        region: DefinitionRegion,
        sort: SortStrategy,
    ) -> None:
        """Add an input port with sorting strategy."""
        ...

    def with_input_auto(
        self, name: str, io_type: IoType, region: DefinitionRegion
    ) -> None:
        """Add an input port with automatic layout."""
        ...

    def with_input_auto_sorted(
        self,
        name: str,
        io_type: IoType,
        region: DefinitionRegion,
        sort: SortStrategy,
    ) -> None:
        """Add an input port with automatic layout and sorting."""
        ...

    def with_output(
        self,
        name: str,
        io_type: IoType,
        layout: LayoutFunction,
        region: DefinitionRegion,
    ) -> None:
        """Add an output port."""
        ...

    def with_output_sorted(
        self,
        name: str,
        io_type: IoType,
        layout: LayoutFunction,
        region: DefinitionRegion,
        sort: SortStrategy,
    ) -> None:
        """Add an output port with sorting strategy."""
        ...

    def with_output_auto(
        self, name: str, io_type: IoType, region: DefinitionRegion
    ) -> None:
        """Add an output port with automatic layout."""
        ...

    def with_output_auto_sorted(
        self,
        name: str,
        io_type: IoType,
        region: DefinitionRegion,
        sort: SortStrategy,
    ) -> None:
        """Add an output port with automatic layout and sorting."""
        ...

    def with_state_mode(self, mode: str) -> None:
        """Set the state mode (e.g., 'lever', 'repeater')."""
        ...

    def validate(self) -> None:
        """Validate the builder configuration."""
        ...

    def build(self) -> "TypedCircuitExecutor":
        """Build the executor."""
        ...

    def build_validated(self) -> "TypedCircuitExecutor":
        """Validate and build the executor."""
        ...

    def input_count(self) -> int:
        """Get the number of inputs."""
        ...

    def output_count(self) -> int:
        """Get the number of outputs."""
        ...

    def input_names(self) -> List[str]:
        """Get input port names."""
        ...

    def output_names(self) -> List[str]:
        """Get output port names."""
        ...

    def __repr__(self) -> str: ...


class TypedCircuitExecutor:
    """Executes typed circuits with structured I/O."""

    @staticmethod
    def from_layout(
        world: MchprsWorld, layout: IoLayout
    ) -> "TypedCircuitExecutor":
        """Create from a simulation world and I/O layout."""
        ...

    @staticmethod
    def from_insign(schematic: Schematic) -> "TypedCircuitExecutor":
        """Create from InSign definitions in a schematic."""
        ...

    def set_state_mode(self, mode: str) -> None:
        """Set the state mode."""
        ...

    def reset(self) -> None:
        """Reset all inputs to default values."""
        ...

    def tick(self, ticks: int) -> None:
        """Advance the simulation by ticks."""
        ...

    def flush(self) -> None:
        """Flush pending changes."""
        ...

    def set_input(self, name: str, value: Value) -> None:
        """Set an input value."""
        ...

    def read_output(self, name: str) -> Any:
        """Read an output value."""
        ...

    def input_names(self) -> List[str]:
        """Get input port names."""
        ...

    def output_names(self) -> List[str]:
        """Get output port names."""
        ...

    def get_layout_info(self) -> Dict[str, Any]:
        """Get detailed layout information."""
        ...

    def execute(
        self, inputs: Dict[str, Any], mode: ExecutionMode
    ) -> Dict[str, Any]:
        """Execute with inputs and return outputs."""
        ...

    def __repr__(self) -> str: ...


# ============================================================================
# Meshing types (requires 'meshing' feature)
# ============================================================================


class ResourcePack:
    """Minecraft resource pack for mesh generation."""

    @staticmethod
    def from_file(path: str) -> "ResourcePack":
        """Load a resource pack from a file path."""
        ...

    @staticmethod
    def from_bytes(data: bytes) -> "ResourcePack":
        """Load a resource pack from bytes."""
        ...

    def stats(self) -> Dict[str, Any]:
        """Get resource pack statistics."""
        ...

    @property
    def blockstate_count(self) -> int:
        """Number of block state definitions."""
        ...

    @property
    def model_count(self) -> int:
        """Number of models."""
        ...

    @property
    def texture_count(self) -> int:
        """Number of textures."""
        ...

    @property
    def namespaces(self) -> List[str]:
        """List of namespaces."""
        ...

    def list_blockstates(self) -> List[str]:
        """List all blockstate names as 'namespace:block_id'."""
        ...

    def list_models(self) -> List[str]:
        """List all model names as 'namespace:model_path'."""
        ...

    def list_textures(self) -> List[str]:
        """List all texture names as 'namespace:texture_path'."""
        ...

    def get_blockstate_json(self, name: str) -> Optional[str]:
        """Get a blockstate definition as a JSON string. Returns None if not found."""
        ...

    def get_model_json(self, name: str) -> Optional[str]:
        """Get a block model as a JSON string. Returns None if not found."""
        ...

    def get_texture_info(self, name: str) -> Optional[Dict[str, Any]]:
        """Get texture info dict with width, height, is_animated, frame_count."""
        ...

    def get_texture_pixels(self, name: str) -> Optional[bytes]:
        """Get raw RGBA8 pixel data for a texture. Returns None if not found."""
        ...

    def add_blockstate_json(self, name: str, json: str) -> None:
        """Add a blockstate definition from a JSON string."""
        ...

    def add_model_json(self, name: str, json: str) -> None:
        """Add a block model from a JSON string."""
        ...

    def add_texture(
        self, name: str, width: int, height: int, pixels: bytes
    ) -> None:
        """Add a texture from raw RGBA8 pixel data."""
        ...

    def __repr__(self) -> str: ...


class MeshConfig:
    """Configuration for mesh generation."""

    def __init__(
        self,
        cull_hidden_faces: bool = True,
        ambient_occlusion: bool = True,
        ao_intensity: float = 0.4,
        biome: Optional[str] = None,
        atlas_max_size: int = 4096,
        cull_occluded_blocks: bool = True,
        greedy_meshing: bool = False,
    ) -> None:
        """Create mesh configuration."""
        ...

    @property
    def cull_hidden_faces(self) -> bool: ...
    @cull_hidden_faces.setter
    def cull_hidden_faces(self, value: bool) -> None: ...

    @property
    def ambient_occlusion(self) -> bool: ...
    @ambient_occlusion.setter
    def ambient_occlusion(self, value: bool) -> None: ...

    @property
    def ao_intensity(self) -> float: ...
    @ao_intensity.setter
    def ao_intensity(self, value: float) -> None: ...

    @property
    def biome(self) -> Optional[str]: ...
    @biome.setter
    def biome(self, value: Optional[str]) -> None: ...

    @property
    def atlas_max_size(self) -> int: ...
    @atlas_max_size.setter
    def atlas_max_size(self, value: int) -> None: ...

    @property
    def cull_occluded_blocks(self) -> bool:
        """Skip blocks fully hidden by opaque neighbors on all 6 sides."""
        ...
    @cull_occluded_blocks.setter
    def cull_occluded_blocks(self, value: bool) -> None: ...

    @property
    def greedy_meshing(self) -> bool:
        """Merge adjacent coplanar faces into larger quads."""
        ...
    @greedy_meshing.setter
    def greedy_meshing(self, value: bool) -> None: ...

    def __repr__(self) -> str: ...


class MeshResult:
    """Result of mesh generation."""

    def save(self, path: str) -> None:
        """Save mesh to a GLB file."""
        ...

    @property
    def glb_data(self) -> bytes:
        """Raw GLB binary data."""
        ...

    @property
    def vertex_count(self) -> int:
        """Number of vertices."""
        ...

    @property
    def triangle_count(self) -> int:
        """Number of triangles."""
        ...

    @property
    def has_transparency(self) -> bool:
        """Whether the mesh has transparent elements."""
        ...

    @property
    def bounds(self) -> List[float]:
        """Mesh bounding box [min_x, min_y, min_z, max_x, max_y, max_z]."""
        ...

    def __repr__(self) -> str: ...


class MultiMeshResult:
    """Result of per-region mesh generation."""

    def get_mesh(self, region_name: str) -> Optional[MeshResult]:
        """Get mesh for a specific region."""
        ...

    def get_all_meshes(self) -> Dict[str, MeshResult]:
        """Get all region meshes."""
        ...

    def save_all(self, prefix: str) -> List[str]:
        """Save all meshes to files. Returns list of paths."""
        ...

    @property
    def region_names(self) -> List[str]:
        """Names of meshed regions."""
        ...

    @property
    def total_vertex_count(self) -> int:
        """Total vertices across all meshes."""
        ...

    @property
    def total_triangle_count(self) -> int:
        """Total triangles across all meshes."""
        ...

    @property
    def mesh_count(self) -> int:
        """Number of meshes."""
        ...

    def __repr__(self) -> str: ...


class ChunkMeshResult:
    """Result of per-chunk mesh generation."""

    def get_mesh(
        self, cx: int, cy: int, cz: int
    ) -> Optional[MeshResult]:
        """Get mesh for a specific chunk."""
        ...

    def get_all_meshes(
        self,
    ) -> Dict[Tuple[int, int, int], MeshResult]:
        """Get all chunk meshes."""
        ...

    def save_all(self, prefix: str) -> List[str]:
        """Save all meshes to files. Returns list of paths."""
        ...

    @property
    def chunk_coordinates(self) -> List[Tuple[int, int, int]]:
        """Coordinates of meshed chunks."""
        ...

    @property
    def total_vertex_count(self) -> int:
        """Total vertices across all meshes."""
        ...

    @property
    def total_triangle_count(self) -> int:
        """Total triangles across all meshes."""
        ...

    @property
    def chunk_count(self) -> int:
        """Number of chunk meshes."""
        ...

    def __repr__(self) -> str: ...


class RawMeshExport:
    """Raw mesh data for custom rendering pipelines."""

    def positions_flat(self) -> List[float]:
        """Get vertex positions as a flat list (x, y, z, x, y, z, ...)."""
        ...

    def normals_flat(self) -> List[float]:
        """Get vertex normals as a flat list."""
        ...

    def uvs_flat(self) -> List[float]:
        """Get texture coordinates as a flat list (u, v, u, v, ...)."""
        ...

    def colors_flat(self) -> List[float]:
        """Get vertex colors as a flat list (r, g, b, a, ...)."""
        ...

    def indices(self) -> List[int]:
        """Get triangle indices."""
        ...

    def texture_rgba(self) -> bytes:
        """Get texture atlas RGBA pixel data."""
        ...

    @property
    def texture_width(self) -> int:
        """Texture atlas width."""
        ...

    @property
    def texture_height(self) -> int:
        """Texture atlas height."""
        ...

    @property
    def vertex_count(self) -> int:
        """Number of vertices."""
        ...

    @property
    def triangle_count(self) -> int:
        """Number of triangles."""
        ...

    def __repr__(self) -> str: ...


# ============================================================================
# Module-level functions
# ============================================================================


def debug_schematic(schematic: Schematic) -> str:
    """Get detailed debug information about a schematic."""
    ...


def debug_json_schematic(schematic: Schematic) -> str:
    """Get schematic information in JSON format."""
    ...


def load_schematic(path: str) -> Schematic:
    """Load a schematic from a file path."""
    ...


def save_schematic(
    schematic: Schematic,
    path: str,
    format: str = "auto",
    version: Optional[str] = None,
) -> None:
    """Save a schematic to a file."""
    ...
