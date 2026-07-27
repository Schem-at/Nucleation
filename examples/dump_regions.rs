fn main() {
    let path = std::env::args().nth(1).unwrap();
    let data = std::fs::read(&path).unwrap();
    let s = nucleation::litematic::from_litematic(&data).unwrap();
    println!("default region blocks: {}", s.default_region.count_blocks());
    println!("named regions: {}", s.other_regions.len());
    for (name, r) in &s.other_regions {
        println!("   {name}: {} blocks", r.count_blocks());
    }
    println!("iter_blocks total: {}", s.iter_blocks().count());
    println!("dimensions: {:?}", s.get_dimensions());
}
