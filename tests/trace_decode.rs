/// Trace exactly what the working decoder saw
#[test]
fn trace_working_decoder() {
    let data = vec![255, 255, 255, 151, 222, 246, 253, 190, 141, 247, 7, 127, 255, 255, 255, 255, 192];
    
    // Invert the data as the working decoder did
    let inverted: Vec<u8> = data.iter().map(|&b| !b).collect();
    
    println!("Inverted bytes:");
    for (i, &b) in inverted.iter().take(10).enumerate() {
        println!("  Byte {}: {:3} = {:08b}", i, b, b);
    }
    
    // The working decoder found start at bit 25
    println!("\nChecking bit 25 for start sentinel (should be 11010):");
    let mut value = 0u8;
    for bit_idx in 0..5 {
        let absolute_bit = 25 + bit_idx;
        let byte_idx = absolute_bit / 8;
        let bit_in_byte = absolute_bit % 8;
        
        let bit = (inverted[byte_idx] >> (7 - bit_in_byte)) & 1;
        value |= bit << bit_idx;
        println!("  Bit {}: {} (from byte {} bit {})", 
                 absolute_bit, bit, byte_idx, bit_in_byte);
    }
    println!("  Value: {:05b} = {}", value, value);
    
    if value == 0b11010 {
        println!("  ✓ Found start sentinel!");
    }
    
    // Now decode from bit 30 (after start sentinel)
    println!("\nDecoding from bit 30:");
    let mut decoded = String::new();
    let mut offset = 30;
    
    // Expected values from working decoder
    let expected = vec![
        (0b00001, '0'),
        (0b00001, '0'),
        (0b00001, '0'),
        (0b00100, '4'),
        (0b00001, '0'),
        (0b00100, '4'),
        (0b00010, '8'),
        (0b11100, '7'),
        (0b10000, '1'),
        (0b01000, '2'),
    ];
    
    for (i, (expected_bits, expected_char)) in expected.iter().enumerate() {
        let mut char_val = 0u8;
        for bit_idx in 0..5 {
            let absolute_bit = offset + bit_idx;
            let byte_idx = absolute_bit / 8;
            let bit_in_byte = absolute_bit % 8;
            
            let bit = (inverted[byte_idx] >> (7 - bit_in_byte)) & 1;
            char_val |= bit << bit_idx;
        }
        
        // Check parity (odd)
        let mut ones = 0;
        for j in 0..5 {
            if (char_val >> j) & 1 == 1 {
                ones += 1;
            }
        }
        let parity_ok = ones % 2 == 1;
        
        let data_bits = char_val & 0x0F;
        let ch = (0x30 + data_bits) as char;
        
        println!("  Char {}: {:05b} => '{}' (expected {:05b} => '{}') {}",
                 i, char_val, ch, expected_bits, expected_char,
                 if parity_ok { "✓" } else { "✗" });
        
        if char_val == *expected_bits {
            decoded.push(ch);
        }
        
        offset += 5;
    }
    
    println!("\nDecoded: {}", decoded);
    assert_eq!(decoded, "0004048712");
}