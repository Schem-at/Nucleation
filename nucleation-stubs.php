<?php

/**
 * Nucleation PHP Extension Stubs
 * Auto-generated stubs for IDE support
 *
 * @version 0.1.35
 * @link https://github.com/Schem-at/Nucleation
 */

/**
 * Simple test function to verify the extension works
 *
 * @return string Welcome message
 */
function nucleation_hello(): string {}

/**
 * Get version information
 *
 * @return array<string, string> Version information
 */
function nucleation_version(): array {}

/**
 * Detect schematic format from binary data
 *
 * @param string $data Binary schematic data
 * @return string Format type: 'litematic', 'schematic', or 'unknown'
 */
function nucleation_detect_format(string $data): string {}

/**
 * Convert between schematic formats
 *
 * @param string $inputData Input schematic data
 * @param string $outputFormat Target format ('litematic' or 'schematic')
 * @return string Converted schematic data
 * @throws Exception On conversion failure
 */
function nucleation_convert_format(string $inputData, string $outputFormat): string {}

/**
 * Create a new schematic
 *
 * @param string $name Schematic name
 * @return \Nucleation\Schematic New schematic instance
 */
function nucleation_create_schematic(string $name): \Nucleation\Schematic {}

/**
 * Load schematic from file path
 *
 * @param string $filePath Path to schematic file
 * @return \Nucleation\Schematic Loaded schematic
 * @throws Exception On file read or parse failure
 */
function nucleation_load_from_file(string $filePath): \Nucleation\Schematic {}

/**
 * Save schematic to file
 *
 * @param \Nucleation\Schematic $schematic Schematic to save
 * @param string $filePath Output file path
 * @param string $format Output format ('litematic' or 'schematic')
 * @return bool Success status
 * @throws Exception On save failure
 */
function nucleation_save_to_file(\Nucleation\Schematic $schematic, string $filePath, string $format): bool {}

namespace Nucleation {
    /**
     * Universal Schematic class for Minecraft schematic manipulation
     */
    class Schematic {
        /**
         * Constructor
         *
         * @param string|null $name Optional schematic name
         */
        public function __construct(?string $name = null) {}

        /**
         * Open a schematic from a URI (local path, file://, or s3://bucket/key.schem).
         *
         * The format is auto-detected. Remote backends such as s3:// require the
         * corresponding store feature (e.g. store-s3) to be enabled at build time;
         * local paths and file:// work with the default build.
         *
         * @param string $uri Local path, file:// URI, or s3:// URI
         * @return \Nucleation\Schematic Opened schematic
         * @throws Exception On read or parse failure
         */
        public static function open(string $uri): \Nucleation\Schematic {}

        /**
         * Save this schematic to a URI (local path, file://, or s3://bucket/key.schem).
         *
         * The output format is inferred from the URI extension. Remote backends such
         * as s3:// require the corresponding store feature (e.g. store-s3) to be
         * enabled at build time; local paths and file:// work with the default build.
         *
         * @param string $uri Local path, file:// URI, or s3:// URI
         * @return void
         * @throws Exception On save failure
         */
        public function save(string $uri): void {}

        /**
         * Load from binary data (auto-detect format)
         *
         * @param string $data Binary schematic data
         * @return bool Success status
         * @throws Exception On parse failure
         */
        public function loadFromData(string $data): bool {}

        /**
         * Load from litematic data
         *
         * @param string $data Litematic binary data
         * @return bool Success status
         * @throws Exception On parse failure
         */
        public function fromLitematic(string $data): bool {}

        /**
         * Load from schematic data
         *
         * @param string $data Schematic binary data
         * @return bool Success status
         * @throws Exception On parse failure
         */
        public function fromSchematic(string $data): bool {}

        /**
         * Export to litematic format
         *
         * @return string Litematic binary data
         * @throws Exception On export failure
         */
        public function toLitematic(): string {}

        /**
         * Export to schematic format
         *
         * @return string Schematic binary data
         * @throws Exception On export failure
         */
        public function toSchematic(): string {}

        /**
         * Set a block at coordinates
         *
         * @param int $x X coordinate
         * @param int $y Y coordinate
         * @param int $z Z coordinate
         * @param string $blockName Block name (e.g., 'minecraft:stone')
         */
        public function setBlock(int $x, int $y, int $z, string $blockName): void {}

        /**
         * Set a block from a block string
         *
         * @param int $x X coordinate
         * @param int $y Y coordinate
         * @param int $z Z coordinate
         * @param string $blockString Block string with properties (e.g., 'minecraft:stairs[facing=north]')
         * @throws Exception On invalid block string
         */
        public function setBlockFromString(int $x, int $y, int $z, string $blockString): void {}

        /**
         * Set a block with properties
         *
         * @param int $x X coordinate
         * @param int $y Y coordinate
         * @param int $z Z coordinate
         * @param string $blockName Block name
         * @param array<string, string> $properties Block properties
         */
        public function setBlockWithProperties(int $x, int $y, int $z, string $blockName, array $properties): void {}

        /**
         * Get block at coordinates
         *
         * @param int $x X coordinate
         * @param int $y Y coordinate
         * @param int $z Z coordinate
         * @return string|null Block name or null if no block
         */
        public function getBlock(int $x, int $y, int $z): ?string {}

        /**
         * Get block with properties
         *
         * @param int $x X coordinate
         * @param int $y Y coordinate
         * @param int $z Z coordinate
         * @return array<string, string>|null Block data with properties or null
         */
        public function getBlockWithProperties(int $x, int $y, int $z): ?array {}

        /**
         * Get schematic dimensions
         *
         * @return array{0: int, 1: int, 2: int} [width, height, length]
         */
        public function getDimensions(): array {}

        /**
         * Get total block count
         *
         * @return int Number of blocks
         */
        public function getBlockCount(): int {}

        /**
         * Get total volume
         *
         * @return int Total volume (width * height * length)
         */
        public function getVolume(): int {}

        /**
         * Get region names
         *
         * @return array<string> List of region names
         */
        public function getRegionNames(): array {}

        /**
         * Get basic schematic information
         *
         * @return array<string, string> Schematic metadata and stats
         */
        public function getInfo(): array {}

        /**
         * Set metadata name
         *
         * @param string $name Schematic name
         */
        public function setMetadataName(string $name): void {}

        /**
         * Get metadata name
         *
         * @return string|null Schematic name
         */
        public function getMetadataName(): ?string {}

        /**
         * Set metadata author
         *
         * @param string $author Author name
         */
        public function setMetadataAuthor(string $author): void {}

        /**
         * Get metadata author
         *
         * @return string|null Author name
         */
        public function getMetadataAuthor(): ?string {}

        /**
         * Set metadata description
         *
         * @param string $description Schematic description
         */
        public function setMetadataDescription(string $description): void {}

        /**
         * Get metadata description
         *
         * @return string|null Schematic description
         */
        public function getMetadataDescription(): ?string {}

        /**
         * Format the schematic as a human-readable string
         *
         * @return string Formatted schematic information
         */
        public function format(): string {}

        /**
         * Format the schematic as JSON
         *
         * @return string JSON representation
         */
        public function formatJson(): string {}

        /**
         * Get debug information
         *
         * @return string Debug information
         */
        public function debugInfo(): string {}

        /**
         * Convert to string representation
         *
         * @return string String representation
         */
        public function __toString(): string {}

        /**
         * Get all blocks as array
         *
         * @return array<array<string, string>> Array of block information with coordinates
         */
        public function getAllBlocks(): array {}

        /**
         * Copy a region from another schematic
         *
         * @param \Nucleation\Schematic $fromSchematic Source schematic
         * @param int $minX Minimum X coordinate
         * @param int $minY Minimum Y coordinate
         * @param int $minZ Minimum Z coordinate
         * @param int $maxX Maximum X coordinate
         * @param int $maxY Maximum Y coordinate
         * @param int $maxZ Maximum Z coordinate
         * @param int $targetX Target X coordinate
         * @param int $targetY Target Y coordinate
         * @param int $targetZ Target Z coordinate
         * @param array<string>|null $excludedBlocks Optional list of blocks to exclude
         * @throws Exception On copy failure
         */
        public function copyRegion(
            \Nucleation\Schematic $fromSchematic,
            int $minX,
            int $minY,
            int $minZ,
            int $maxX,
            int $maxY,
            int $maxZ,
            int $targetX,
            int $targetY,
            int $targetZ,
            ?array $excludedBlocks = null
        ): void {}

        /**
         * Compute the structural fingerprint of this schematic as a hex string.
         *
         * @param string $preset Fingerprint preset: "exact", "shape",
         *   "structural", "redstone_computational", "redstone",
         *   "redstone_survival".
         * @return string Hex-encoded fingerprint
         * @throws Exception On unknown preset
         */
        public function fingerprint(string $preset): string {}

        /**
         * Compute the structural signature of this schematic as JSON.
         *
         * @param string $preset Fingerprint preset (see fingerprint())
         * @return string JSON signature
         * @throws Exception On unknown preset
         */
        public function signature(string $preset): string {}

        /**
         * Translation-invariant fuzzy footprint distance to another schematic
         * (0.0 = identical occupancy shape).
         *
         * @param \Nucleation\Schematic $other Other schematic
         * @param string $preset Fingerprint preset (see fingerprint())
         * @return float Footprint distance
         * @throws Exception On unknown preset
         */
        public function footprintDistance(\Nucleation\Schematic $other, string $preset): float {}

        /**
         * Returns true if this schematic shares the same fingerprint as another.
         *
         * @param \Nucleation\Schematic $other Other schematic
         * @param string $preset Fingerprint preset (see fingerprint())
         * @return bool Whether the two are duplicates
         */
        public function isDuplicateOf(\Nucleation\Schematic $other, string $preset): bool {}

        /**
         * Compute the structural diff between this schematic and another.
         *
         * @param \Nucleation\Schematic $other Target schematic
         * @param string $preset Diff preset
         * @param int|null $costAdd Optional cost override for additions
         * @param int|null $costDelete Optional cost override for deletions
         * @param int|null $costChange Optional cost override for changes
         * @param int|null $costSwap Optional cost override for swaps
         * @param string|null $symmetry Optional symmetry: "none", "yaw",
         *   "yaw_mirror", "octahedral", "octahedral_full".
         * @return \Nucleation\Diff The computed diff
         * @throws Exception On unknown preset or symmetry
         */
        public function diff(
            \Nucleation\Schematic $other,
            string $preset,
            ?int $costAdd = null,
            ?int $costDelete = null,
            ?int $costChange = null,
            ?int $costSwap = null,
            ?string $symmetry = null
        ): \Nucleation\Diff {}
    }

    /**
     * Result of a structural diff between two schematics.
     */
    class Diff {
        /**
         * Reconstruct a Diff from its JSON representation.
         *
         * @param string $json JSON produced by toJson()
         * @return \Nucleation\Diff The reconstructed diff
         * @throws Exception On parse error
         */
        public static function fromJson(string $json): \Nucleation\Diff {}

        /**
         * The edit distance (total cost) of this diff.
         *
         * @return int Edit distance
         */
        public function distance(): int {}

        /**
         * The alignment support / confidence of this diff.
         *
         * @return float Support value
         */
        public function support(): float {}

        /**
         * Serialize this diff to its full JSON representation.
         *
         * @return string JSON
         */
        public function toJson(): string {}

        /**
         * Serialize this diff to its compact summary JSON.
         *
         * @return string Summary JSON
         */
        public function summaryJson(): string {}

        /**
         * A new schematic containing only the blocks added in this diff.
         *
         * @return \Nucleation\Schematic Added blocks
         */
        public function added(): \Nucleation\Schematic {}

        /**
         * A new schematic containing only the blocks removed in this diff.
         *
         * @return \Nucleation\Schematic Removed blocks
         */
        public function removed(): \Nucleation\Schematic {}

        /**
         * A new schematic containing only the blocks changed in this diff.
         *
         * @return \Nucleation\Schematic Changed blocks
         */
        public function changed(): \Nucleation\Schematic {}

        /**
         * A new schematic containing only the blocks swapped in this diff.
         *
         * @return \Nucleation\Schematic Swapped blocks
         */
        public function swapped(): \Nucleation\Schematic {}

        /**
         * A new schematic with marker blocks summarizing this diff.
         *
         * @return \Nucleation\Schematic Marker schematic
         */
        public function markers(): \Nucleation\Schematic {}

        /**
         * Render a diff overlay on top of an "after" GLB buffer, returning a
         * new GLB buffer. Requires the meshing feature.
         *
         * @param string $afterGlb The "after" GLB binary data
         * @return string New GLB binary data
         * @throws Exception On overlay error
         */
        public function toOverlayGlb(string $afterGlb): string {}
    }
}