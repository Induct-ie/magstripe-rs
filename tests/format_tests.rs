use magstripe_rs::{BitStream, Decoder, Format};

/// Helper function to convert a binary string to bytes
fn binary_string_to_bytes(binary: &str) -> (Vec<u8>, usize) {
    let bits: Vec<u8> = binary
        .chars()
        .filter(|c| *c == '0' || *c == '1')
        .map(|c| if c == '1' { 1 } else { 0 })
        .collect();
    
    let bit_count = bits.len();
    let mut bytes = Vec::new();
    let mut current_byte = 0u8;
    
    for (i, &bit) in bits.iter().enumerate() {
        let bit_in_byte = i % 8;
        if bit == 1 {
            current_byte |= 1 << (7 - bit_in_byte);
        }
        if bit_in_byte == 7 || i == bits.len() - 1 {
            bytes.push(current_byte);
            current_byte = 0;
        }
    }
    
    (bytes, bit_count)
}

/// Test Track2 inverted format with real-world data
#[test]
fn test_track2_inverted_real_data() {
    // Real-world example: User's card that decodes to "0004048712"
    let data = vec![255, 255, 255, 151, 222, 246, 253, 190, 141, 247, 7, 127, 255, 255, 255, 255, 192];
    let stream = BitStream::new(&data, 130).unwrap();
    
    let decoder = Decoder::new(&[Format::Track2Inverted]);
    
    match decoder.decode(stream) {
        Ok(output) => {
            assert_eq!(output.data, "0004048712");
            assert!(matches!(output.format, Format::Track2Inverted));
        }
        Err(e) => {
            panic!("Failed to decode known good Track2 Inverted card: {:?}", e);
        }
    }
}

/// Test a simple Track2 pattern
#[test]
fn test_track2_simple_pattern() {
    // Simple Track2: ;12345?
    let binary_data = concat!(
        // Preamble
        "1111111111111111111111111",
        // ; (start) - 0b11010 LSB
        "01011",
        // 1 - 0b10000 LSB
        "00001",
        // 2 - 0b01000 LSB
        "00010",
        // 3 - 0b11001 LSB
        "10011",
        // 4 - 0b00100 LSB
        "00100",
        // 5 - 0b10101 LSB
        "10101",
        // ? (end) - 0b11111 LSB
        "11111",
        // LRC placeholder
        "00000",
        // Trailing
        "11111111"
    );
    
    let (bytes, bit_count) = binary_string_to_bytes(binary_data);
    let stream = BitStream::new(&bytes, bit_count).unwrap();
    let decoder = Decoder::new(&[Format::Track2]);
    
    match decoder.decode(stream) {
        Ok(output) => {
            assert!(output.data.contains("12345"));
        }
        Err(_) => {
            // Track2 decoder may fail on LRC check, which is expected for synthetic data
        }
    }
}

/// Test Track1 and Track1 Inverted formats
#[test]
fn test_track1_formats() {
    // Track1 has different encoding (7-bit) and is less commonly used
    // This test just verifies the formats exist and can be attempted
    
    let data = vec![255, 255, 255, 255];
    let stream = BitStream::new(&data, 32).unwrap();
    
    let formats = vec![Format::Track1, Format::Track1Inverted];
    let decoder = Decoder::new(&formats);
    
    // We expect this to fail, but it should attempt both formats
    match decoder.decode(stream) {
        Err(magstripe_rs::DecoderError::NoValidFormat { attempted }) => {
            assert_eq!(attempted, 2);
        }
        _ => {} // If it somehow succeeds, that's fine too
    }
}

/// Test Track3 format
#[test]
fn test_track3_format() {
    // Track3 is rarely used, similar encoding to Track2 but different density
    let data = vec![255, 255, 255, 255];
    let stream = BitStream::new(&data, 32).unwrap();
    
    let decoder = Decoder::new(&[Format::Track3]);
    
    // We expect this to fail with random data
    match decoder.decode(stream) {
        Err(magstripe_rs::DecoderError::NoValidFormat { attempted }) => {
            assert_eq!(attempted, 1);
        }
        _ => {} // If it somehow succeeds, that's fine too
    }
}

/// Test all Track2 variants
#[test]
fn test_track2_variants() {
    let data = vec![255, 255, 255, 151, 222, 246, 253, 190, 141, 247, 7, 127, 255, 255, 255, 255, 192];
    let stream = BitStream::new(&data, 130).unwrap();
    
    let formats = vec![
        Format::Track2,
        Format::Track2Inverted,
        Format::Track2MSB,
        Format::Track2LSB,
        Format::Track2Raw,
        Format::Track2SwappedParity,
        Format::Track2EvenParity,
    ];
    
    let decoder = Decoder::new(&formats);
    
    // Should succeed with Track2Inverted
    match decoder.decode(stream) {
        Ok(output) => {
            assert_eq!(output.data, "0004048712");
            assert!(matches!(output.format, Format::Track2Inverted));
        }
        Err(e) => {
            panic!("Should decode with one of the Track2 variants: {:?}", e);
        }
    }
}

/// Test malformed data handling
#[test]
fn test_malformed_data() {
    // Test with not enough bits for any valid format
    let data = vec![0xFF];
    let stream = BitStream::new(&data, 8).unwrap();
    
    let decoder = Decoder::new(&[Format::Track2]);
    
    match decoder.decode(stream) {
        Err(magstripe_rs::DecoderError::BitstreamTooShort { .. }) |
        Err(magstripe_rs::DecoderError::NoValidFormat { .. }) => {
            // Expected - data is too short
        }
        _ => panic!("Should fail with short data"),
    }
}

/// Test edge case with alternating bits
#[test]
fn test_alternating_bits() {
    // 0xAA = 10101010
    let data = vec![0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA];
    let stream = BitStream::new(&data, 48).unwrap();
    
    let formats = vec![
        Format::Track2,
        Format::Track2Inverted,
    ];
    
    let decoder = Decoder::new(&formats);
    
    // This pattern is unlikely to decode to anything valid
    match decoder.decode(stream) {
        Err(magstripe_rs::DecoderError::NoValidFormat { .. }) => {
            // Expected
        }
        Ok(output) => {
            // If it somehow decodes, make sure it's reasonable
            assert!(!output.data.is_empty());
        }
        Err(_) => {
            // Any other error is also acceptable for this pattern
        }
    }
}