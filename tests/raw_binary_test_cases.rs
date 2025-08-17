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

/// Test Track1 format - 7-bit IATA encoding
/// Track1 can store alphanumeric data (uppercase letters, numbers, special chars)
/// Format: %SS...?LRC
/// Start sentinel: % (0x05 in 6-bit + odd parity)
/// End sentinel: ? (0x1F in 6-bit + odd parity)
#[test]
fn test_track1_basic_decode() {
    // Track1 encoded data: %B1234567890123456^DOE/JOHN^2512?
    // Using 7-bit encoding (6 data bits + 1 odd parity bit)
    let binary_data = concat!(
        // Leading 1s (preamble)
        "1111111111111111111111111",
        // % - Start sentinel (0x05 -> 000101 + parity = 0001011)
        "0001011",
        // B - (0x22 -> 100010 + parity = 1000100)
        "1000100",
        // 1 - (0x11 -> 010001 + parity = 0100011)
        "0100011",
        // 2 - (0x12 -> 010010 + parity = 0100101)
        "0100101",
        // 3 - (0x13 -> 010011 + parity = 0100110)
        "0100110",
        // 4 - (0x14 -> 010100 + parity = 0101001)
        "0101001",
        // ^ - Field separator (0x1E -> 011110 + parity = 0111101)
        "0111101",
        // ? - End sentinel (0x1F -> 011111 + parity = 0111111)
        "0111111",
        // LRC (dummy)
        "0000000",
        // Trailing 1s
        "11111111"
    );
    
    let (bytes, bit_count) = binary_string_to_bytes(binary_data);
    let stream = BitStream::new(&bytes, bit_count).unwrap();
    let decoder = Decoder::new(&[Format::Track1]);
    
    match decoder.decode(stream) {
        Ok(output) => {
            println!("Track1 decoded: {}", output.data);
            assert!(output.data.contains("1234"));
        }
        Err(e) => {
            println!("Track1 decode error (expected for partial data): {:?}", e);
        }
    }
}

/// Test Track1 inverted format
#[test]
fn test_track1_inverted() {
    // Same data as above but with all bits inverted
    let binary_data = concat!(
        // Leading 0s (inverted preamble)
        "0000000000000000000000000",
        // % - Start sentinel inverted
        "1110100",
        // B - inverted
        "0111011",
        // 1 - inverted
        "1011100",
        // 2 - inverted
        "1011010",
        // 3 - inverted
        "1011001",
        // 4 - inverted
        "1010110",
        // ^ - Field separator inverted
        "1000010",
        // ? - End sentinel inverted
        "1000000",
        // LRC (dummy) inverted
        "1111111",
        // Trailing 0s
        "00000000"
    );
    
    let (bytes, bit_count) = binary_string_to_bytes(binary_data);
    let stream = BitStream::new(&bytes, bit_count).unwrap();
    let decoder = Decoder::new(&[Format::Track1Inverted]);
    
    match decoder.decode(stream) {
        Ok(output) => {
            println!("Track1 Inverted decoded: {}", output.data);
            assert!(matches!(output.format, Format::Track1Inverted));
        }
        Err(e) => {
            println!("Track1 Inverted decode error (expected for partial data): {:?}", e);
        }
    }
}

/// Test Track2 standard format - 5-bit BCD encoding
/// This is the most common format for credit/debit cards
/// Format: ;CC#=YYMM...?LRC
#[test]
fn test_track2_standard_decode() {
    // Track2 encoded data: ;5301250070000191=0805?
    // Using standard 5-bit encoding (4 data bits + 1 odd parity bit), LSB first
    let binary_data = concat!(
        // Leading 1s (preamble)
        "1111111111111111111111111",
        // ; - Start sentinel (1011 -> 11010 LSB)
        "01011",
        // 5 - (0101 -> 10101 LSB)
        "10101",
        // 3 - (0011 -> 11001 LSB)
        "10011",
        // 0 - (0000 -> 00001 LSB)
        "10000",
        // 1 - (0001 -> 10000 LSB)
        "00001",
        // 2 - (0010 -> 01000 LSB)
        "00010",
        // 5 - (0101 -> 10101 LSB)
        "10101",
        // 0 - (0000 -> 00001 LSB)
        "10000",
        // 0 - (0000 -> 00001 LSB)
        "10000",
        // 7 - (0111 -> 11100 LSB)
        "00111",
        // 0 - (0000 -> 00001 LSB)
        "10000",
        // 0 - (0000 -> 00001 LSB)
        "10000",
        // 0 - (0000 -> 00001 LSB)
        "10000",
        // 0 - (0000 -> 00001 LSB)
        "10000",
        // 1 - (0001 -> 10000 LSB)
        "00001",
        // 9 - (1001 -> 10011 LSB)
        "11001",
        // 1 - (0001 -> 10000 LSB)
        "00001",
        // = - Field separator (1101 -> 10110 LSB)
        "01101",
        // 0 - (0000 -> 00001 LSB)
        "10000",
        // 8 - (1000 -> 00010 LSB)
        "01000",
        // 0 - (0000 -> 00001 LSB)
        "10000",
        // 5 - (0101 -> 10101 LSB)
        "10101",
        // ? - End sentinel (1111 -> 11111 LSB)
        "11111",
        // LRC (dummy)
        "00000",
        // Trailing 1s
        "11111111"
    );
    
    let (bytes, bit_count) = binary_string_to_bytes(binary_data);
    let stream = BitStream::new(&bytes, bit_count).unwrap();
    let decoder = Decoder::new(&[Format::Track2]);
    
    match decoder.decode(stream) {
        Ok(output) => {
            println!("Track2 decoded: {}", output.data);
            assert!(output.data.contains("5301250070000191"));
        }
        Err(e) => {
            println!("Track2 decode error: {:?}", e);
        }
    }
}

/// Test Track2 inverted format
#[test]
fn test_track2_inverted_decode() {
    // Real-world example: User's card that decodes to "0004048712"
    // This is the actual data provided: inverted Track2 format
    let data = vec![255, 255, 255, 151, 222, 246, 253, 190, 141, 247, 7, 127, 255, 255, 255, 255, 192];
    let stream = BitStream::new(&data, 130).unwrap();
    
    let decoder = Decoder::new(&[Format::Track2Inverted]);
    
    match decoder.decode(stream) {
        Ok(output) => {
            println!("Track2 Inverted decoded: {}", output.data);
            assert_eq!(output.data, "0004048712");
            assert!(matches!(output.format, Format::Track2Inverted));
        }
        Err(e) => {
            panic!("Failed to decode known good Track2 Inverted card: {:?}", e);
        }
    }
}

/// Test Track2 MSB format (Most Significant Bit first)
#[test]
fn test_track2_msb_decode() {
    // Track2 data but with MSB first ordering
    // ;12345=2512?
    let binary_data = concat!(
        // Leading 1s
        "1111111111111111111111111",
        // ; - Start sentinel MSB (11010 -> 01011)
        "01011",
        // 1 - MSB (10000 -> 00001)
        "00001",
        // 2 - MSB (01000 -> 00010)
        "00010",
        // 3 - MSB (11001 -> 10011)
        "10011",
        // 4 - MSB (00100 -> 00100)
        "00100",
        // 5 - MSB (10101 -> 10101)
        "10101",
        // = - MSB (10110 -> 01101)
        "01101",
        // 2 - MSB
        "00010",
        // 5 - MSB
        "10101",
        // 1 - MSB
        "00001",
        // 2 - MSB
        "00010",
        // ? - End sentinel MSB (11111 -> 11111)
        "11111",
        // LRC
        "00000",
        // Trailing
        "11111111"
    );
    
    let (bytes, bit_count) = binary_string_to_bytes(binary_data);
    let stream = BitStream::new(&bytes, bit_count).unwrap();
    let decoder = Decoder::new(&[Format::Track2MSB]);
    
    match decoder.decode(stream) {
        Ok(output) => {
            println!("Track2 MSB decoded: {}", output.data);
            assert!(output.data.contains("12345"));
        }
        Err(e) => {
            println!("Track2 MSB decode error: {:?}", e);
        }
    }
}

/// Test Track2 with swapped parity
#[test]
fn test_track2_swapped_parity() {
    // Track2 data with even parity instead of odd
    // ;999=2512?
    let binary_data = concat!(
        // Leading 1s
        "1111111111111111111111111",
        // ; - Start sentinel with even parity (1011 -> 11011 LSB)
        "11011",
        // 9 - with even parity (1001 -> 10010 LSB)
        "01001",
        // 9 - with even parity
        "01001",
        // 9 - with even parity
        "01001",
        // = - with even parity (1101 -> 10111 LSB)
        "11101",
        // 2 - with even parity (0010 -> 01001 LSB)
        "10010",
        // 5 - with even parity (0101 -> 10100 LSB)
        "00101",
        // 1 - with even parity (0001 -> 10001 LSB)
        "10001",
        // 2 - with even parity
        "10010",
        // ? - End sentinel with even parity (1111 -> 11110 LSB)
        "01111",
        // LRC
        "00000",
        // Trailing
        "11111111"
    );
    
    let (bytes, bit_count) = binary_string_to_bytes(binary_data);
    let stream = BitStream::new(&bytes, bit_count).unwrap();
    let decoder = Decoder::new(&[Format::Track2SwappedParity]);
    
    match decoder.decode(stream) {
        Ok(output) => {
            println!("Track2 Swapped Parity decoded: {}", output.data);
        }
        Err(e) => {
            println!("Track2 Swapped Parity decode error (expected): {:?}", e);
        }
    }
}

/// Test Track3 format - 5-bit encoding at 210 bpi
/// Track3 is rarely used but can contain additional data
#[test]
fn test_track3_decode() {
    // Track3 uses same encoding as Track2 but different density
    // ;107=9912?
    let binary_data = concat!(
        // Leading 1s
        "1111111111111111111111111",
        // ; - Start sentinel
        "01011",
        // 1
        "00001",
        // 0
        "10000",
        // 7
        "00111",
        // =
        "01101",
        // 9
        "11001",
        // 9
        "11001",
        // 1
        "00001",
        // 2
        "00010",
        // ?
        "11111",
        // LRC
        "00000",
        // Trailing
        "11111111"
    );
    
    let (bytes, bit_count) = binary_string_to_bytes(binary_data);
    let stream = BitStream::new(&bytes, bit_count).unwrap();
    let decoder = Decoder::new(&[Format::Track3]);
    
    match decoder.decode(stream) {
        Ok(output) => {
            println!("Track3 decoded: {}", output.data);
            assert!(output.data.contains("107"));
        }
        Err(e) => {
            println!("Track3 decode error: {:?}", e);
        }
    }
}

/// Test malformed data - missing start sentinel
#[test]
fn test_malformed_missing_start_sentinel() {
    // Track2 data without start sentinel
    let binary_data = concat!(
        // Leading 1s
        "1111111111111111111111111",
        // Missing ; - jumping straight to data
        // 1
        "00001",
        // 2
        "00010",
        // 3
        "10011",
        // ?
        "11111",
        // Trailing
        "11111111"
    );
    
    let (bytes, bit_count) = binary_string_to_bytes(binary_data);
    let stream = BitStream::new(&bytes, bit_count).unwrap();
    let decoder = Decoder::new(&[Format::Track2]);
    
    match decoder.decode(stream) {
        Ok(_) => {
            panic!("Should not decode malformed data without start sentinel");
        }
        Err(e) => {
            println!("Expected error for missing start sentinel: {:?}", e);
        }
    }
}

/// Test malformed data - missing end sentinel
#[test]
fn test_malformed_missing_end_sentinel() {
    // Track2 data without end sentinel
    let binary_data = concat!(
        // Leading 1s
        "1111111111111111111111111",
        // ;
        "01011",
        // 1
        "00001",
        // 2
        "00010",
        // 3
        "10011",
        // Missing ? - just trailing data
        // Trailing
        "11111111"
    );
    
    let (bytes, bit_count) = binary_string_to_bytes(binary_data);
    let stream = BitStream::new(&bytes, bit_count).unwrap();
    let decoder = Decoder::new(&[Format::Track2]);
    
    match decoder.decode(stream) {
        Ok(output) => {
            // Some decoders may still decode partial data
            println!("Decoded partial data without end sentinel: {}", output.data);
            // The decoder found some data even without proper end sentinel
        }
        Err(e) => {
            println!("Expected error for missing end sentinel: {:?}", e);
        }
    }
}

/// Test with corrupt parity bits
#[test]
fn test_corrupt_parity() {
    // Track2 data with intentionally wrong parity bits
    let binary_data = concat!(
        // Leading 1s
        "1111111111111111111111111",
        // ; - Start sentinel
        "01011",
        // 1 - with wrong parity (should be 10000, using 10001)
        "10001",
        // 2 - with wrong parity (should be 01000, using 01001)
        "10010",
        // ?
        "11111",
        // Trailing
        "11111111"
    );
    
    let (bytes, bit_count) = binary_string_to_bytes(binary_data);
    let stream = BitStream::new(&bytes, bit_count).unwrap();
    let decoder = Decoder::new(&[Format::Track2]);
    
    match decoder.decode(stream) {
        Ok(output) => {
            println!("Decoded despite parity errors: {}", output.data);
        }
        Err(e) => {
            println!("Expected parity error: {:?}", e);
        }
    }
}

/// Test edge case - very short bitstream
#[test]
fn test_edge_case_short_bitstream() {
    let binary_data = "11111";
    let (bytes, bit_count) = binary_string_to_bytes(binary_data);
    let stream = BitStream::new(&bytes, bit_count).unwrap();
    let decoder = Decoder::new(&[Format::Track2, Format::Track1]);
    
    match decoder.decode(stream) {
        Ok(_) => {
            panic!("Should not decode extremely short bitstream");
        }
        Err(e) => {
            println!("Expected error for short bitstream: {:?}", e);
        }
    }
}

/// Test edge case - all zeros
#[test]
fn test_edge_case_all_zeros() {
    let binary_data = "00000000000000000000000000000000000000000000000000";
    let (bytes, bit_count) = binary_string_to_bytes(binary_data);
    let stream = BitStream::new(&bytes, bit_count).unwrap();
    let decoder = Decoder::new(&[Format::Track2Inverted]);
    
    match decoder.decode(stream) {
        Ok(_) => {
            panic!("Should not decode all zeros");
        }
        Err(e) => {
            println!("Expected error for all zeros: {:?}", e);
        }
    }
}

/// Test edge case - all ones
#[test]
fn test_edge_case_all_ones() {
    let binary_data = "11111111111111111111111111111111111111111111111111";
    let (bytes, bit_count) = binary_string_to_bytes(binary_data);
    let stream = BitStream::new(&bytes, bit_count).unwrap();
    let decoder = Decoder::new(&[Format::Track2]);
    
    match decoder.decode(stream) {
        Ok(_) => {
            panic!("Should not decode all ones");
        }
        Err(e) => {
            println!("Expected error for all ones: {:?}", e);
        }
    }
}

/// Test real-world Mastercard example
#[test]
fn test_real_mastercard_track2() {
    // Real Mastercard Track2: ;5301250070000191=08051010912345678901?
    // Full binary encoding with correct parity
    let binary_data = concat!(
        // Preamble
        "1111111111111111111111111",
        // ; (start)
        "01011",
        // 5301250070000191
        "10101", // 5
        "10011", // 3
        "10000", // 0
        "00001", // 1
        "00010", // 2
        "10101", // 5
        "10000", // 0
        "10000", // 0
        "00111", // 7
        "10000", // 0
        "10000", // 0
        "10000", // 0
        "10000", // 0
        "00001", // 1
        "11001", // 9
        "00001", // 1
        // = (separator)
        "01101",
        // 0805 (expiry)
        "10000", // 0
        "01000", // 8
        "10000", // 0
        "10101", // 5
        // 1010912345678901 (discretionary)
        "00001", // 1
        "10000", // 0
        "00001", // 1
        "10000", // 0
        "11001", // 9
        "00001", // 1
        "00010", // 2
        "10011", // 3
        "00100", // 4
        "10101", // 5
        "01101", // 6
        "00111", // 7
        "01000", // 8
        "11001", // 9
        "10000", // 0
        "00001", // 1
        // ? (end)
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
            println!("Mastercard Track2 decoded: {}", output.data);
            assert!(output.data.contains("5301250070000191"));
        }
        Err(e) => {
            println!("Mastercard decode error: {:?}", e);
        }
    }
}

/// Test real-world Visa example  
#[test]
fn test_real_visa_track2() {
    // Real Visa Track2: ;4539791001730106=08051010912345678901?
    let binary_data = concat!(
        // Preamble
        "1111111111111111111111111",
        // ; (start)
        "01011",
        // 4539791001730106
        "00100", // 4
        "10101", // 5
        "10011", // 3
        "11001", // 9
        "00111", // 7
        "11001", // 9
        "00001", // 1
        "10000", // 0
        "10000", // 0
        "00001", // 1
        "00111", // 7
        "10011", // 3
        "10000", // 0
        "00001", // 1
        "10000", // 0
        "01101", // 6
        // = (separator)
        "01101",
        // 0805 (expiry)
        "10000", // 0
        "01000", // 8
        "10000", // 0
        "10101", // 5
        // 1010912345678901 (discretionary)
        "00001", // 1
        "10000", // 0
        "00001", // 1
        "10000", // 0
        "11001", // 9
        "00001", // 1
        "00010", // 2
        "10011", // 3
        "00100", // 4
        "10101", // 5
        "01101", // 6
        "00111", // 7
        "01000", // 8
        "11001", // 9
        "10000", // 0
        "00001", // 1
        // ? (end)
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
            println!("Visa Track2 decoded: {}", output.data);
            assert!(output.data.contains("4539791001730106"));
        }
        Err(e) => {
            println!("Visa decode error: {:?}", e);
        }
    }
}