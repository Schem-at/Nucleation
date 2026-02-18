use nucleation::formats::manager::get_manager;

#[test]
#[ignore] // Requires local file: /Users/harrison/Downloads/55 3x3.zip
fn test_55_3x3_world_detect() {
    let data = std::fs::read("/Users/harrison/Downloads/55 3x3.zip").unwrap();
    println!("File size: {}", data.len());

    let manager = get_manager();
    let manager = manager.lock().unwrap();

    let detected = manager.detect_format(&data);
    println!("Detected format: {:?}", detected);

    match manager.read(&data) {
        Ok(schematic) => {
            println!("Read OK! Regions: {:?}", schematic.get_region_names());
        }
        Err(e) => {
            panic!("Read failed: {}", e);
        }
    }
}
