use nucleation::litematic;
use nucleation::schematic;
use nucleation::BlockState;
use nucleation::UniversalSchematic;

// Mocking SchematicWrapper from src/wasm/schematic.rs
pub struct SchematicWrapper(pub(crate) UniversalSchematic);

impl SchematicWrapper {
    pub fn new() -> Self {
        SchematicWrapper(UniversalSchematic::new("Default".to_string()))
    }

    pub fn from_data(&mut self, data: &[u8]) -> Result<(), String> {
        if litematic::is_litematic(data) {
            self.from_litematic(data)
        } else if schematic::is_schematic(data) {
            self.from_schematic(data)
        } else {
            Err("Unknown or unsupported schematic format".to_string())
        }
    }

    pub fn from_litematic(&mut self, data: &[u8]) -> Result<(), String> {
        self.0 = litematic::from_litematic(data)
            .map_err(|e| format!("Litematic parsing error: {}", e))?;
        Ok(())
    }

    pub fn to_litematic(&self) -> Result<Vec<u8>, String> {
        litematic::to_litematic(&self.0).map_err(|e| format!("Litematic conversion error: {}", e))
    }

    pub fn from_schematic(&mut self, data: &[u8]) -> Result<(), String> {
        self.0 = schematic::from_schematic(data)
            .map_err(|e| format!("Schematic parsing error: {}", e))?;
        Ok(())
    }

    pub fn to_schematic(&self) -> Result<Vec<u8>, String> {
        schematic::to_schematic(&self.0).map_err(|e| format!("Schematic conversion error: {}", e))
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block_name: &str) {
        self.0.set_block_str(x, y, z, block_name);
    }

    pub fn get_dimensions(&self) -> Vec<i32> {
        let (x, y, z) = self.0.get_dimensions();
        vec![x, y, z]
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<String> {
        self.0.get_block(x, y, z).map(|b| b.name.clone())
    }

    pub fn blocks(&self) -> Vec<BlockState> {
        self.0.get_blocks()
    }
}

#[test]
fn test_schematic_roundtrip_preserves_blocks() {
    // 1. Create a new schematic
    let mut schematic = SchematicWrapper::new();

    // 2. Set some blocks
    schematic.set_block(0, 0, 0, "minecraft:glass");
    schematic.set_block(1, 0, 0, "minecraft:glass");
    schematic.set_block(0, 1, 0, "minecraft:glass");
    schematic.set_block(1, 1, 0, "minecraft:glass");
    schematic.set_block(0, 0, 1, "minecraft:glass");
    schematic.set_block(1, 0, 1, "minecraft:glass");
    schematic.set_block(0, 1, 1, "minecraft:glass");
    schematic.set_block(1, 1, 1, "minecraft:glass");

    // 3. Serialize to .schem format
    let serialized = schematic.to_schematic().expect("Failed to serialize");

    // 4. Create a new schematic and load the serialized data
    let mut loaded = SchematicWrapper::new();
    loaded
        .from_data(&serialized)
        .expect("Failed to deserialize");

    // 5. Verify dimensions are correct
    let dims = loaded.get_dimensions();
    assert_eq!(dims, vec![2, 2, 2]);

    // 6. Verify blocks are preserved
    let block = loaded.get_block(0, 0, 0).expect("Block not found at 0,0,0");
    assert_eq!(block, "minecraft:glass", "Block at 0,0,0 should be glass");

    let block = loaded.get_block(1, 1, 1).expect("Block not found at 1,1,1");
    assert_eq!(block, "minecraft:glass", "Block at 1,1,1 should be glass");

    // 7. Verify block count
    let blocks = loaded.blocks();
    let non_air_blocks: Vec<_> = blocks
        .iter()
        .filter(|b| b.name != "minecraft:air")
        .collect();
    assert_eq!(non_air_blocks.len(), 8);
}

// test making a schematic adding a couple glass blocks -> to_schematic -> from_schematic -> add diamond block -> to_schematic -> from_schematic -> check all blocks are there
#[test]
fn test_schematic_modification_roundtrip() {
    // 1. Create a new schematic
    let mut schematic = SchematicWrapper::new();
    // 2. Set some glass blocks
    schematic.set_block(0, 0, 0, "minecraft:glass");
    schematic.set_block(1, 0, 0, "minecraft:glass");
    schematic.set_block(0, 1, 0, "minecraft:glass");
    schematic.set_block(1, 1, 0, "minecraft:glass");
    // 3. Serialize to .schem format
    let serialized = schematic.to_schematic().expect("Failed to serialize");
    // 4. Create a new schematic and load the serialized data
    let mut loaded = SchematicWrapper::new();
    loaded
        .from_data(&serialized)
        .expect("Failed to deserialize");
    // 5. Add a diamond block
    loaded.set_block(0, 0, 1, "minecraft:diamond_block");
    // 6. Serialize again
    let re_serialized = loaded.to_schematic().expect("Failed to re-serialize");
    // 7. Load into another schematic
    let mut final_loaded = SchematicWrapper::new();
    final_loaded
        .from_data(&re_serialized)
        .expect("Failed to re-deserialize");
    // 8. Verify all blocks are present
    let block = final_loaded.get_block(0, 0, 0).expect(
        "
Block not found at 0,0,0",
    );
    assert_eq!(
        block, "minecraft:glass",
        "Block at 0,0,
0 should be glass"
    );
    let block = final_loaded
        .get_block(1, 1, 0)
        .expect("Block not found at 1,1,0");
    assert_eq!(block, "minecraft:glass", "Block at 1,1,0 should be glass");
    let block = final_loaded
        .get_block(0, 0, 1)
        .expect("Block not found at 0,0,1");
    assert_eq!(
        block, "minecraft:diamond_block",
        "Block at 0,0,1 should be diamond block"
    );
}
