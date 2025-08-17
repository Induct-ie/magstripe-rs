use clap::Parser;
use magstripe_rs::{BitStream, Decoder, Format};
use std::process;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

/// Magnetic stripe decoder CLI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Byte array as JSON-like format, e.g., "[255, 255, 255, 151, 222, 246]"
    #[arg(value_name = "BYTES")]
    bytes: String,
    
    /// Number of bits to decode (defaults to 8 * number of bytes)
    #[arg(short, long)]
    bits: Option<usize>,
    
    /// Enable verbose output with tracing logs (set RUST_LOG=debug for more details)
    #[arg(short, long)]
    verbose: bool,
    
    /// Try all known formats (by default, tries common ones)
    #[arg(short = 'a', long)]
    all_formats: bool,
    
    /// Only try specific format(s), can be specified multiple times
    #[arg(short = 'f', long, value_enum)]
    format: Vec<FormatArg>,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum FormatArg {
    Track1,
    Track1Inverted,
    Track2,
    Track2Inverted,
    Track2Msb,
    Track2Lsb,
    Track2Raw,
    Track2SwappedParity,
    Track2EvenParity,
    Track3,
}

impl FormatArg {
    fn to_format(&self) -> Format {
        match self {
            FormatArg::Track1 => Format::Track1,
            FormatArg::Track1Inverted => Format::Track1Inverted,
            FormatArg::Track2 => Format::Track2,
            FormatArg::Track2Inverted => Format::Track2Inverted,
            FormatArg::Track2Msb => Format::Track2MSB,
            FormatArg::Track2Lsb => Format::Track2LSB,
            FormatArg::Track2Raw => Format::Track2Raw,
            FormatArg::Track2SwappedParity => Format::Track2SwappedParity,
            FormatArg::Track2EvenParity => Format::Track2EvenParity,
            FormatArg::Track3 => Format::Track3,
        }
    }
}

fn parse_bytes(input: &str) -> Result<Vec<u8>, String> {
    // Remove whitespace and brackets
    let cleaned = input.trim().trim_start_matches('[').trim_end_matches(']');
    
    // Parse as comma-separated values
    let bytes: Result<Vec<u8>, _> = cleaned
        .split(',')
        .map(|s| {
            let trimmed = s.trim();
            // Try parsing as decimal first
            trimmed.parse::<u8>()
                .or_else(|_| {
                    // Try hex with 0x prefix
                    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
                        u8::from_str_radix(&trimmed[2..], 16)
                    } else {
                        Err("Invalid number format".parse::<u8>().unwrap_err())
                    }
                })
        })
        .collect();
    
    bytes.map_err(|e| format!("Failed to parse bytes: {}", e))
}

fn main() {
    let args = Args::parse();
    
    // Setup tracing
    if args.verbose {
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::from_default_env()
                    .add_directive("magstripe_rs=debug".parse().unwrap())
            )
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::from_default_env()
                    .add_directive("magstripe_rs=info".parse().unwrap())
            )
            .init();
    }
    
    // Parse the byte array
    let bytes = match parse_bytes(&args.bytes) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!("\nExpected format: [255, 255, 151, ...] or [0xFF, 0xFF, 0x97, ...]");
            process::exit(1);
        }
    };
    
    // Determine bit count
    let bit_count = args.bits.unwrap_or(bytes.len() * 8);
    
    if bit_count > bytes.len() * 8 {
        eprintln!("Error: Bit count {} exceeds available bits {}", 
                  bit_count, bytes.len() * 8);
        process::exit(1);
    }
    
    info!("Decoding {} bits from {} bytes", bit_count, bytes.len());
    
    // Create bitstream
    let stream = match BitStream::new(&bytes, bit_count) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error creating bitstream: {:?}", e);
            process::exit(1);
        }
    };
    
    // Determine which formats to try
    let formats: Vec<Format> = if !args.format.is_empty() {
        // Use specific formats requested
        args.format.iter().map(|f| f.to_format()).collect()
    } else if args.all_formats {
        // Try all known formats
        vec![
            Format::Track2,
            Format::Track2Inverted,
            Format::Track2MSB,
            Format::Track2LSB,
            Format::Track2Raw,
            Format::Track2SwappedParity,
            Format::Track2EvenParity,
            Format::Track1,
            Format::Track1Inverted,
            Format::Track3,
        ]
    } else {
        // Default: try most common formats
        vec![
            Format::Track2,
            Format::Track2Inverted,
            Format::Track1,
            Format::Track1Inverted,
        ]
    };
    
    info!("Trying {} format(s)", formats.len());
    
    // Create decoder and decode
    let decoder = Decoder::new(&formats);
    
    match decoder.decode(stream) {
        Ok(output) => {
            println!("\n✓ Successfully decoded!");
            println!("Format: {:?}", output.format);
            println!("Data:   {}", output.data);
            
            if args.verbose {
                println!("\nFormat details:");
                match output.format {
                    Format::Track2 | Format::Track2Inverted | Format::Track2MSB | 
                    Format::Track2LSB | Format::Track2Raw | Format::Track2SwappedParity | 
                    Format::Track2EvenParity | Format::Track3 => {
                        println!("  Encoding: 5-bit (4 data + 1 parity)");
                        println!("  Character set: 0-9, :, ;, <, =, >, ?");
                    }
                    Format::Track1 | Format::Track1Inverted => {
                        println!("  Encoding: 7-bit (6 data + 1 parity)");
                        println!("  Character set: Alphanumeric (64 characters)");
                    }
                    _ => {}
                }
            }
        }
        Err(e) => {
            error!("Failed to decode: {:?}", e);
            eprintln!("\n✗ Decoding failed: {}", e);
            
            if !args.verbose {
                eprintln!("\nHint: Use -v flag for detailed debug output");
                eprintln!("      Set RUST_LOG=trace for even more details");
            }
            
            process::exit(1);
        }
    }
}