
use xixi_core::GameConfig;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  Convert mode: {} convert <input.json> <output.rkyv>", args[0]);
        eprintln!("  Validate mode: {} validate <input.rkyv>", args[0]);
        eprintln!("  Deploy mode: {} deploy <input.rkyv> (zero-copy validation)", args[0]);
        std::process::exit(1);
    }

    match args[1].as_str() {
        "convert" if args.len() == 4 => {
            let json_str = std::fs::read_to_string(&args[2]).unwrap();
            let config = GameConfig::from_json(&json_str).unwrap();
            let rkyv_bytes = config.to_rkyv();
            std::fs::write(&args[3], rkyv_bytes).unwrap();
            println!("Successfully converted {} to {}", args[2], args[3]);
        }
        "validate" if args.len() == 3 => {
            let rkyv_bytes = std::fs::read(&args[2]).unwrap();
            let config = GameConfig::from_rkyv(&rkyv_bytes);
            
            println!("Validating GameConfig fields from {}...", args[2]);
            println!("Major nodes: {}", config.major_adj.len());
            println!("Major-minor mapping: {}", config.major_minor_adj.len());
            println!("Major descriptions: {}", config.major_desc.len());
            println!("Minor rewards: {}", config.minor_desc.len());
            println!("Enemies: {}", config.enemy_data.len());
            println!("Initial stats: {:?}", config.init_stat);
            println!("Level up requirements: {}", config.levelup_desc.len());
            
            println!("All fields validated successfully");
        }
        "deploy" if args.len() == 3 => {
            // Create properly aligned buffer
            use rkyv::AlignedVec;
            let mut file = std::fs::File::open(&args[2]).unwrap();
            let mut buffer = AlignedVec::new();
            buffer.extend_from_reader(&mut file).unwrap();

            // Validate with proper alignment
            let archived = rkyv::check_archived_root::<GameConfig>(&buffer)
                .expect("Failed to validate archived data");
            
            println!("Zero-copy validation of {}...", args[2]);
            println!("Major nodes: {}", archived.major_adj.len());
            println!("Major-minor mapping: {}", archived.major_minor_adj.len());
            println!("Major descriptions: {}", archived.major_desc.len());
            println!("Minor rewards: {}", archived.minor_desc.len());
            println!("Enemies: {}", archived.enemy_data.len());
            println!("Initial stats: {:?}", archived.init_stat);
            println!("Level up requirements: {}", archived.levelup_desc.len());
            
            println!("All fields validated with zero-copy successfully");
        }
        _ => {
            eprintln!("Invalid arguments");
            std::process::exit(1);
        }
    }
}