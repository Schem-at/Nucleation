//! Schematic Builder CLI - Convert text templates to Minecraft schematics
//!
//! Usage:
//!   schematic-builder < input.txt > output.litematic
//!   cat circuit.txt | schematic-builder > circuit.litematic
//!   echo "─→─" | schematic-builder -o repeater.litematic
//!
//! The tool reads template text from stdin and outputs a Litematic schematic file.
//! Standard Unicode palette is loaded by default.

use nucleation::SchematicBuilder;
use std::io::{self, Read, Write};

fn print_help() {
    eprintln!(
        r#"
Schematic Builder CLI - Convert text templates to Minecraft schematics

USAGE:
    schematic-builder [OPTIONS]

OPTIONS:
    -o, --output <FILE>     Output file (default: stdout)
    -i, --input <FILE>      Input file (default: stdin)
    -n, --name <NAME>       Schematic name (default: "schematic")
    -f, --format <FORMAT>   Output format: litematic, schem (default: litematic)
    --no-palette            Don't load standard Unicode palette
    -h, --help              Print this help message

EXAMPLES:
    # Read from stdin, write to stdout
    echo "─→─" | schematic-builder > repeater.litematic

    # Read from file, write to file
    schematic-builder -i circuit.txt -o circuit.litematic

    # Use cat to pipe
    cat my_circuit.txt | schematic-builder -o output.litematic

    # Specify schematic name
    echo "█\n─" | schematic-builder -n "MyCircuit" -o circuit.litematic

TEMPLATE FORMAT:
    # Layer 0
    █████
    █████

    # Layer 1
    ─→→→─

    [palette]
    # Optional: override standard palette characters
    X = minecraft:diamond_block

STANDARD PALETTE:
    The standard Unicode palette is loaded by default, including:
    - Wire: ─ │ ╋ (and corners, T-junctions)
    - Repeaters: → ← ↑ ↓ (1-tick), ⇒ ⇐ ⇑ ⇓ (2-tick), etc.
    - Comparators: ▷ ◁ △ ▽ (compare), ▶ ◀ ▲ ▼ (subtract)
    - Torches: * ⚡ ○
    - Blocks: █ ▓ ▒ ░
    - Air: _ (space) ·

    Use --no-palette to start with an empty palette.
"#
    );
}

struct Config {
    input_file: Option<String>,
    output_file: Option<String>,
    name: String,
    format: String,
    use_palette: bool,
}

impl Config {
    fn from_args() -> Result<Self, String> {
        let mut args = std::env::args().skip(1);
        let mut config = Config {
            input_file: None,
            output_file: None,
            name: "schematic".to_string(),
            format: "litematic".to_string(),
            use_palette: true,
        };

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => {
                    print_help();
                    std::process::exit(0);
                }
                "-i" | "--input" => {
                    config.input_file =
                        Some(args.next().ok_or("Missing value for --input".to_string())?);
                }
                "-o" | "--output" => {
                    config.output_file = Some(
                        args.next()
                            .ok_or("Missing value for --output".to_string())?,
                    );
                }
                "-n" | "--name" => {
                    config.name = args.next().ok_or("Missing value for --name".to_string())?;
                }
                "-f" | "--format" => {
                    config.format = args
                        .next()
                        .ok_or("Missing value for --format".to_string())?;
                    if config.format != "litematic" && config.format != "schem" {
                        return Err(format!("Invalid format: {}", config.format));
                    }
                }
                "--no-palette" => {
                    config.use_palette = false;
                }
                _ => {
                    return Err(format!("Unknown argument: {}", arg));
                }
            }
        }

        Ok(config)
    }
}

fn read_input(config: &Config) -> Result<String, String> {
    let mut input = String::new();

    if let Some(ref file_path) = config.input_file {
        std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read input file '{}': {}", file_path, e))
    } else {
        io::stdin()
            .read_to_string(&mut input)
            .map_err(|e| format!("Failed to read from stdin: {}", e))?;
        Ok(input)
    }
}

fn write_output(config: &Config, data: Vec<u8>) -> Result<(), String> {
    if let Some(ref file_path) = config.output_file {
        std::fs::write(file_path, data)
            .map_err(|e| format!("Failed to write output file '{}': {}", file_path, e))
    } else {
        io::stdout()
            .write_all(&data)
            .map_err(|e| format!("Failed to write to stdout: {}", e))
    }
}

fn main() {
    // Parse arguments
    let config = match Config::from_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!("\nUse --help for usage information");
            std::process::exit(1);
        }
    };

    // Read input
    let input = match read_input(&config) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    if input.trim().is_empty() {
        eprintln!("Error: Input is empty");
        std::process::exit(1);
    }

    // Build schematic
    let schematic = match build_schematic(&input, &config) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error building schematic: {}", e);
            std::process::exit(1);
        }
    };

    // Convert to bytes
    let bytes = match config.format.as_str() {
        "litematic" => match nucleation::litematic::to_litematic(&schematic) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error converting to Litematic: {}", e);
                std::process::exit(1);
            }
        },
        "schem" => match schematic.to_schematic() {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error converting to Sponge Schematic: {}", e);
                std::process::exit(1);
            }
        },
        _ => unreachable!(),
    };

    // Write output
    if let Err(e) = write_output(&config, bytes) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    // Print success message to stderr (so it doesn't interfere with stdout)
    if config.output_file.is_some() {
        eprintln!(
            "✅ Schematic created successfully: {}",
            config.output_file.as_ref().unwrap()
        );
    } else {
        eprintln!("✅ Schematic written to stdout");
    }
}

fn build_schematic(input: &str, config: &Config) -> Result<nucleation::UniversalSchematic, String> {
    // Check if input looks like a template (has [palette] section or # comments)
    let is_template =
        input.contains("[palette]") || input.lines().any(|l| l.trim().starts_with('#'));

    if is_template {
        // Parse as template
        let builder = SchematicBuilder::from_template(input)?;

        if !config.use_palette {
            // Template parsing creates a new builder, so we need to handle this differently
            // For now, just use the template as-is
            eprintln!("Warning: --no-palette is ignored when using template format");
        }

        builder.name(config.name.clone()).build()
    } else {
        // Parse as simple layers (one layer per line, or blank line = new layer)
        let builder = if config.use_palette {
            SchematicBuilder::new()
        } else {
            SchematicBuilder::empty()
        };

        let mut layers = Vec::new();
        let mut current_layer = Vec::new();

        for line in input.lines() {
            let trimmed = line.trim_end();
            if trimmed.is_empty() {
                if !current_layer.is_empty() {
                    layers.push(current_layer);
                    current_layer = Vec::new();
                }
            } else {
                current_layer.push(trimmed.to_string());
            }
        }

        if !current_layer.is_empty() {
            layers.push(current_layer);
        }

        if layers.is_empty() {
            return Err("No layers found in input".to_string());
        }

        // Convert to the format expected by layers()
        let layer_refs: Vec<Vec<&str>> = layers
            .iter()
            .map(|layer| layer.iter().map(|s| s.as_str()).collect())
            .collect();

        let layer_slice_refs: Vec<&[&str]> = layer_refs.iter().map(|v| v.as_slice()).collect();

        builder
            .name(config.name.clone())
            .layers(&layer_slice_refs)
            .build()
    }
}
