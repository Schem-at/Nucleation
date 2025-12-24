use blockpedia::all_blocks;

fn main() {
    let mut blocks = all_blocks();
    if let Some(block) = blocks.next() {
        println!("Inspecting block: {:?}", block.id);
        // Uncomment lines below one by one to check field existence if needed
        // println!("Transparent: {}", block.transparent);
        // println!("BoundingBox: {:?}", block.bounding_box);
    }

    // Print first 5 blocks to see what we have
    for block in all_blocks().take(5) {
        println!("{:?}", block.id);
    }
}
