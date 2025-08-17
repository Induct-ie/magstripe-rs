# magstripe-rs

A robust Rust library and CLI tool for decoding magnetic stripe card data from raw binary streams. Supports multiple track formats and encoding schemes commonly found in magnetic stripe cards.

## Features

- **Multiple Track Format Support**
  - Track 1 (IATA format, 7-bit alphanumeric)
  - Track 2 (ABA format, 5-bit numeric)
  - Track 3 (THRIFT format, 5-bit numeric)
  
- **Encoding Variants**
  - Standard and inverted bit patterns
  - LSB-first and MSB-first bit ordering
  - Configurable parity (odd/even)
  - Raw mode for non-standard cards

- **Robust Decoding**
  - Automatic format detection
  - Start/end sentinel detection
  - Parity checking
  - LRC (Longitudinal Redundancy Check) validation

- **Flexible Input**
  - Decode from raw byte arrays
  - Support for partial/damaged data
  - Configurable bit stream lengths

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
magstripe-rs = "0.1.0"
```

## Library Usage

### Basic Decoding

```rust
use magstripe_rs::{BitStream, Decoder, Format};

// Raw magnetic stripe data
let data = vec![255, 255, 255, 151, 222, 246, 253, 190, 141, 247, 7, 127, 255, 255, 255, 255, 192];
let bit_count = 130;

// Create a bit stream
let stream = BitStream::new(&data, bit_count).unwrap();

// Set up decoder with desired formats
let formats = vec![
    Format::Track2,
    Format::Track2Inverted,
];
let decoder = Decoder::new(&formats);

// Decode the data
match decoder.decode(stream) {
    Ok(output) => {
        println!("Format: {:?}", output.format);
        println!("Data: {}", output.data);
    }
    Err(e) => {
        eprintln!("Decode error: {:?}", e);
    }
}
```

### Custom Format Specification

```rust
use magstripe_rs::{Format, CustomSpec, Decoder, BitStream};

let custom = CustomSpec {
    bits_per_char: 5,
    start_sentinel: Some(0b11010),
    end_sentinel: Some(0b11111),
    char_map: None, // Use default numeric mapping
};

let decoder = Decoder::new(&[Format::Custom(custom)]);
```

## CLI Usage

### Basic Usage

```bash
# Decode with automatic format detection
magstripe-decode "[255, 255, 255, 151, 222, 246, 253, 190, 141, 247, 7, 127, 255, 255, 255, 255, 192]" -b 130

# Verbose output with tracing
magstripe-decode "[255, 255, 255, 151, 222, 246, 253, 190, 141, 247, 7, 127, 255, 255, 255, 255, 192]" -b 130 -v

# Try specific formats
magstripe-decode "[255, 255, 255, 151, 222, 246, 253, 190, 141, 247, 7, 127, 255, 255, 255, 255, 192]" -b 130 -f Track2Inverted
```

### Command Line Options

- `-b, --bits <count>`: Number of bits to process from the input
- `-f, --format <format>`: Specific format to try (default: tries all)
- `-v, --verbose`: Enable verbose output with tracing

### Available Formats

- `Track1`: Standard Track 1 (7-bit IATA)
- `Track1Inverted`: Track 1 with inverted bits
- `Track2`: Standard Track 2 (5-bit ABA)
- `Track2Inverted`: Track 2 with inverted bits
- `Track2MSB`: Track 2 with MSB-first bit order
- `Track2LSB`: Track 2 with LSB-first bit order
- `Track2Raw`: Track 2 without sentinel checking
- `Track2SwappedParity`: Track 2 with swapped parity bits
- `Track2EvenParity`: Track 2 with even parity
- `Track3`: Standard Track 3 format

## Track Format Details

### Track 1 (IATA)
- 210 bits per inch (bpi)
- 7-bit character encoding (6 data + 1 parity)
- Alphanumeric character set
- Start sentinel: `%` (0x05)
- End sentinel: `?` (0x1F)
- Format: `%[data]?[LRC]`

### Track 2 (ABA)
- 75 bpi
- 5-bit character encoding (4 data + 1 parity)
- Numeric only (0-9, special chars)
- Start sentinel: `;` (0x0B)
- End sentinel: `?` (0x0F)
- Format: `;[account]=[expiry][discretionary]?[LRC]`

### Track 3 (THRIFT)
- 210 bpi
- 5-bit character encoding (4 data + 1 parity)
- Numeric character set
- Similar format to Track 2

## Character Encoding

### Track 2 Character Map (5-bit)

| Character | Binary (LSB) | Hex  | Decimal |
|-----------|-------------|------|---------|
| 0         | 00001       | 0x01 | 1       |
| 1         | 10000       | 0x10 | 16      |
| 2         | 01000       | 0x08 | 8       |
| 3         | 11001       | 0x19 | 25      |
| 4         | 00100       | 0x04 | 4       |
| 5         | 10101       | 0x15 | 21      |
| 6         | 01101       | 0x0D | 13      |
| 7         | 11100       | 0x1C | 28      |
| 8         | 00010       | 0x02 | 2       |
| 9         | 10011       | 0x13 | 19      |
| ;         | 11010       | 0x1A | 26      |
| =         | 10110       | 0x16 | 22      |
| ?         | 11111       | 0x1F | 31      |

## Error Handling

The decoder provides detailed error information:

```rust
pub enum DecoderError {
    InvalidBitStream,
    BitstreamTooShort { bit_count: usize, minimum_required: usize },
    NoStartSentinel,
    NoEndSentinel,
    ParityError { position: usize },
    LrcCheckFailed,
    NoFormatsProvided,
    NoValidFormat { attempted: usize },
    InvalidCharacterValue { value: u8, position: usize },
}
```

## Testing

The library includes comprehensive tests for various card formats:

```bash
# Run all tests
cargo test

# Run with verbose output
RUST_LOG=trace cargo test

# Run specific test
cargo test test_track2_inverted_decode
```

## Examples

### Decoding a Real Card

```rust
// Example: Card that reads as inverted Track 2
let data = vec![255, 255, 255, 151, 222, 246, 253, 190, 141, 247, 7, 127, 255, 255, 255, 255, 192];
let stream = BitStream::new(&data, 130).unwrap();

let decoder = Decoder::new(&[Format::Track2Inverted]);
let output = decoder.decode(stream).unwrap();

assert_eq!(output.data, "0004048712");
```

### Handling Unknown Formats

```rust
// Try all common formats
let formats = vec![
    Format::Track1,
    Format::Track1Inverted,
    Format::Track2,
    Format::Track2Inverted,
    Format::Track2MSB,
    Format::Track3,
];

let decoder = Decoder::new(&formats);
match decoder.decode(stream) {
    Ok(output) => println!("Success with format: {:?}", output.format),
    Err(DecoderError::NoValidFormat { attempted }) => {
        println!("Tried {} formats, none worked", attempted);
    }
    Err(e) => println!("Other error: {:?}", e),
}
```

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for bugs and feature requests.

## License

This project is licensed under the Mozilla Public License Version 2.0 - see the LICENSE file for details.

## Safety

This library is designed for educational and legitimate testing purposes only. Always ensure you have proper authorization before reading or decoding magnetic stripe cards.

## Acknowledgments

- Based on ISO/IEC 7811 standards for magnetic stripe cards
- Inspired by various open-source magnetic stripe decoding projects
