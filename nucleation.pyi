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
    
    def with_property(self, key: str, value: str) -> 'BlockState':
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
        """
        Create a new empty schematic.
        
        Args:
            name: Optional name for the schematic (default: "Default")
        """
        ...
    
    def test(self) -> str:
        """Test method to verify the class is working."""
        ...
    
    def from_data(self, data: bytes) -> None:
        """
        Load schematic from raw bytes. Auto-detects format.
        
        Args:
            data: Raw bytes of a .litematic or .schematic file
        
        Raises:
            ValueError: If format is not recognized or invalid
        """
        ...
    
    def from_litematic(self, data: bytes) -> None:
        """
        Load schematic from Litematica format bytes.
        
        Args:
            data: Raw bytes of a .litematic file
        
        Raises:
            ValueError: If data is invalid
        """
        ...
    
    def to_litematic(self) -> bytes:
        """
        Export schematic to Litematica format.
        
        Returns:
            Bytes that can be saved as a .litematic file
        
        Raises:
            IOError: If export fails
        """
        ...
    
    def from_schematic(self, data: bytes) -> None:
        """
        Load schematic from WorldEdit/Sponge format bytes.
        
        Args:
            data: Raw bytes of a .schematic/.schem file
        
        Raises:
            ValueError: If data is invalid
        """
        ...
    
    def to_schematic(self) -> bytes:
        """
        Export schematic to WorldEdit/Sponge format.
        
        Returns:
            Bytes that can be saved as a .schem file
        
        Raises:
            IOError: If export fails
        """
        ...
    
    def set_block(self, x: int, y: int, z: int, block_name: str) -> bool:
        """
        Set a block at the given position using block string notation.
        
        Supports bracket notation: "minecraft:lever[powered=true,facing=north]"
        
        Args:
            x, y, z: Block coordinates
            block_name: Block name with optional properties
        
        Returns:
            True if successful
        """
        ...
    
    def set_block_in_region(
        self, region_name: str, x: int, y: int, z: int, block_name: str
    ) -> bool:
        """
        Set a block in a specific region.
        
        Args:
            region_name: Name of the region
            x, y, z: Block coordinates
            block_name: Block name with optional properties
        
        Returns:
            True if successful
        """
        ...
    
    def clear_cache(self) -> None:
        """Clear the internal block state cache."""
        ...
    
    def cache_info(self) -> Tuple[int, int]:
        """
        Get cache statistics.
        
        Returns:
            Tuple of (cache_size, cache_hits)
        """
        ...
    
    def set_block_from_string(self, x: int, y: int, z: int, block_string: str) -> None:
        """
        Set a block using full string notation (alternative to set_block).
        
        Args:
            x, y, z: Block coordinates
            block_string: Block name with optional properties
        
        Raises:
            ValueError: If block string is invalid
        """
        ...
    
    def set_block_with_properties(
        self, x: int, y: int, z: int, block_name: str, properties: Dict[str, str]
    ) -> None:
        """
        Set a block with explicit properties dictionary.
        
        Args:
            x, y, z: Block coordinates
            block_name: Block name (e.g., "minecraft:lever")
            properties: Dictionary of property key-value pairs
        """
        ...
    
    def copy_region(
        self,
        from_schematic: 'Schematic',
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
        """
        Copy a region from another schematic.
        
        Args:
            from_schematic: Source schematic to copy from
            min_x, min_y, min_z: Minimum coordinates of region
            max_x, max_y, max_z: Maximum coordinates of region
            target_x, target_y, target_z: Target position in this schematic
            excluded_blocks: Optional list of block names to exclude
        
        Raises:
            RuntimeError: If copy operation fails
        """
        ...
    
    def get_block(self, x: int, y: int, z: int) -> Optional[BlockState]:
        """
        Get the block state at the given position.
        
        Args:
            x, y, z: Block coordinates
        
        Returns:
            BlockState if a block exists, None if air or out of bounds
        """
        ...
    
    def get_block_string(self, x: int, y: int, z: int) -> Optional[str]:
        """
        Get block as formatted string with properties.
        
        Example: "minecraft:lever[powered=true,facing=north]"
        
        Args:
            x, y, z: Block coordinates
        
        Returns:
            Formatted block string, or None if no block exists
        """
        ...
    
    def get_palette(self) -> List[BlockState]:
        """
        Get the palette (unique block types) for the default region.
        
        Returns:
            List of BlockState objects representing each unique block type
        """
        ...
    
    def get_block_entity(self, x: int, y: int, z: int) -> Optional[Dict[str, Any]]:
        """
        Get block entity (tile entity) data at position.
        
        Args:
            x, y, z: Block coordinates
        
        Returns:
            Dictionary with 'id', 'position', and 'nbt' keys, or None
        """
        ...
    
    def get_all_block_entities(self) -> List[Dict[str, Any]]:
        """
        Get all block entities in the schematic.
        
        Returns:
            List of block entity dictionaries
        """
        ...
    
    def get_all_blocks(self) -> List[Dict[str, Any]]:
        """
        Get all blocks with their positions and properties.
        
        Returns:
            List of dicts with 'x', 'y', 'z', 'name', 'properties' keys
        """
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
        """
        Get schematic data organized into chunks.
        
        Args:
            chunk_width: Width of each chunk
            chunk_height: Height of each chunk
            chunk_length: Length of each chunk
            strategy: Loading strategy - "distance_to_camera", "top_down", 
                     "bottom_up", "center_outward", or "random"
            camera_x, camera_y, camera_z: Camera position (for distance strategy)
        
        Returns:
            List of chunk dictionaries with 'chunk_x', 'chunk_y', 'chunk_z', 'blocks'
        """
        ...
    
    @property
    def dimensions(self) -> Tuple[int, int, int]:
        """
        Get schematic dimensions.
        
        Returns:
            Tuple of (width, height, depth)
        """
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
    
    def debug_info(self) -> str:
        """Get debug information about the schematic."""
        ...
    
    def create_simulation_world(self) -> 'MchprsWorld':
        """
        Create a redstone simulation world from this schematic.
        
        Requires the 'simulation' feature to be enabled.
        
        Returns:
            MchprsWorld for redstone circuit simulation
        
        Raises:
            RuntimeError: If simulation initialization fails
        """
        ...
    
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...

class MchprsWorld:
    """
    Redstone circuit simulation world powered by MCHPRS.
    
    Available when the 'simulation' feature is enabled.
    """
    
    def on_use_block(self, x: int, y: int, z: int) -> None:
        """
        Simulate right-clicking a block (e.g., toggling a lever).
        
        Args:
            x, y, z: Block coordinates
        """
        ...
    
    def tick(self, ticks: int) -> None:
        """
        Advance the simulation by a number of ticks.
        
        Args:
            ticks: Number of ticks to simulate
        """
        ...
    
    def flush(self) -> None:
        """Flush pending simulation changes to the world."""
        ...
    
    def is_lit(self, x: int, y: int, z: int) -> bool:
        """
        Check if a redstone lamp is lit at the given position.
        
        Args:
            x, y, z: Block coordinates
        
        Returns:
            True if the block is a lit lamp
        """
        ...
    
    def get_lever_power(self, x: int, y: int, z: int) -> bool:
        """
        Get the power state of a lever.
        
        Args:
            x, y, z: Block coordinates
        
        Returns:
            True if lever is powered (on)
        """
        ...
    
    def get_redstone_power(self, x: int, y: int, z: int) -> int:
        """
        Get redstone power level at position (0-15).
        
        Note: This reads the 'power' property from redstone wire blocks.
        For other blocks, use is_lit() or get_lever_power() instead.
        
        Args:
            x, y, z: Block coordinates
        
        Returns:
            Power level (0-15)
        """
        ...
    
    def sync_to_schematic(self) -> None:
        """
        Sync current simulation state back to the schematic.
        
        Updates all block states (power levels, lever states, lamp states, etc.)
        from the simulation back to the UniversalSchematic.
        """
        ...
    
    def get_schematic(self) -> Schematic:
        """
        Get the underlying schematic.
        
        Call sync_to_schematic() first to get the latest simulation state.
        
        Returns:
            Reference to the schematic
        """
        ...
    
    def into_schematic(self) -> Schematic:
        """
        Consume the world and return the schematic with synced state.
        
        Automatically syncs the simulation state before returning.
        
        Returns:
            The schematic with simulation state applied
        """
        ...
    
    def __repr__(self) -> str: ...

def debug_schematic(schematic: Schematic) -> str:
    """
    Get detailed debug information about a schematic.
    
    Args:
        schematic: The schematic to debug
    
    Returns:
        Formatted debug string
    """
    ...

def debug_json_schematic(schematic: Schematic) -> str:
    """
    Get schematic information in JSON format.
    
    Args:
        schematic: The schematic to debug
    
    Returns:
        JSON formatted string
    """
    ...

def load_schematic(path: str) -> Schematic:
    """
    Load a schematic from a file path.
    
    Args:
        path: Path to .litematic or .schematic file
    
    Returns:
        Loaded schematic
    
    Raises:
        IOError: If file cannot be read
        ValueError: If file format is invalid
    """
    ...

def save_schematic(schematic: Schematic, path: str, format: str = "auto") -> None:
    """
    Save a schematic to a file.
    
    Args:
        schematic: The schematic to save
        path: Output file path
        format: Format to use - "litematic", "schematic", or "auto" (default)
               Auto-detects from file extension
    
    Raises:
        IOError: If file cannot be written
        ValueError: If format is invalid
    """
    ...
