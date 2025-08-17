use magstripe_rs::{BitStream, Decoder, Format};

fn analyze_track2_pattern(data: &[u8], bit_count: usize) {
    println!("\nAnalyzing as Track 2 (5-bit groups, LSB first):");
    
    let mut offset = 0;
    let mut char_num = 0;
    
    while offset + 5 <= bit_count {
        let mut value = 0u8;
        
        // Extract 5 bits LSB-first
        for bit_idx in 0..5 {
            let absolute_bit = offset + bit_idx;
            let byte_idx = absolute_bit / 8;
            let bit_in_byte = absolute_bit % 8;
            
            if byte_idx < data.len() {
                let bit = (data[byte_idx] >> (7 - bit_in_byte)) & 1;
                value |= bit << bit_idx;
            }
        }
        
        // Decode the character
        let data_bits = value & 0x0F;
        let parity_bit = (value >> 4) & 1;
        
        // Check odd parity
        let mut ones = 0;
        for i in 0..5 {
            if (value >> i) & 1 == 1 {
                ones += 1;
            }
        }
        let parity_ok = ones % 2 == 1;
        
        // Decode character
        let ascii = 0x30 + data_bits;
        let ch = if ascii <= 0x3F { ascii as char } else { '?' };
        
        println!("  Char {:2}: {:05b} (data={:04b}, p={}) => '{}' (0x{:02X}) {}",
                 char_num, value, data_bits, parity_bit, ch, data_bits,
                 if parity_ok { "✓" } else { "✗ parity" });
        
        // Check for sentinels
        if value == 0b11010 {
            println!("    ^ Start sentinel (;)");
        } else if value == 0b11111 {
            println!("    ^ End sentinel (?)");
        }
        
        offset += 5;
        char_num += 1;
        
        if char_num >= 20 {
            println!("  ... (showing first 20 characters)");
            break;
        }
    }
}

#[test]
fn debug_test_card() {
    let data = vec![255, 255, 255, 151, 222, 246, 253, 190, 141, 247, 7, 127, 255, 255, 255, 255, 192];
    let bit_count = 130;
    
    println!("Original data (first 8 bytes in binary):");
    for (i, &byte) in data.iter().take(8).enumerate() {
        println!("  Byte {:2}: {:3} = {:08b}", i, byte, byte);
    }
    
    // Analyze as Track 2
    analyze_track2_pattern(&data, bit_count);
    
    // Try inverted
    println!("\n=== INVERTED DATA ===");
    let inverted: Vec<u8> = data.iter().map(|&b| !b).collect();
    
    println!("Inverted data (first 8 bytes in binary):");
    for (i, &byte) in inverted.iter().take(8).enumerate() {
        println!("  Byte {:2}: {:3} = {:08b}", i, byte, byte);
    }
    
    analyze_track2_pattern(&inverted, bit_count);
    
    // Look for the start sentinel in different positions
    println!("\n=== SEARCHING FOR START SENTINEL ===");
    
    // Track 2 start sentinel is 11010 (;)
    let track2_start = 0b11010;
    
    for offset in 0..50 {
        let mut value = 0u8;
        
        // Extract 5 bits at this offset
        for bit_idx in 0..5 {
            let absolute_bit = offset + bit_idx;
            let byte_idx = absolute_bit / 8;
            let bit_in_byte = absolute_bit % 8;
            
            if byte_idx < data.len() {
                let bit = (data[byte_idx] >> (7 - bit_in_byte)) & 1;
                value |= bit << bit_idx;
            }
        }
        
        if value == track2_start {
            println!("Found Track2 start sentinel at bit offset {}", offset);
        }
        
        // Check inverted too
        value = 0;
        for bit_idx in 0..5 {
            let absolute_bit = offset + bit_idx;
            let byte_idx = absolute_bit / 8;
            let bit_in_byte = absolute_bit % 8;
            
            if byte_idx < inverted.len() {
                let bit = (inverted[byte_idx] >> (7 - bit_in_byte)) & 1;
                value |= bit << bit_idx;
            }
        }
        
        if value == track2_start {
            println!("Found Track2 start sentinel in INVERTED data at bit offset {}", offset);
            
            // Let's decode from here
            println!("\nDecoding from offset {}:", offset);
            let mut decoded = String::new();
            let mut pos = offset + 5; // Skip sentinel
            
            while pos + 5 <= bit_count {
                let mut char_val = 0u8;
                for bit_idx in 0..5 {
                    let absolute_bit = pos + bit_idx;
                    let byte_idx = absolute_bit / 8;
                    let bit_in_byte = absolute_bit % 8;
                    
                    if byte_idx < inverted.len() {
                        let bit = (inverted[byte_idx] >> (7 - bit_in_byte)) & 1;
                        char_val |= bit << bit_idx;
                    }
                }
                
                // Check for end sentinel
                if char_val == 0b11111 {
                    println!("Found end sentinel at position {}", pos);
                    break;
                }
                
                let data_bits = char_val & 0x0F;
                let ch = (0x30 + data_bits) as char;
                decoded.push(ch);
                
                pos += 5;
                
                if decoded.len() >= 40 {
                    break;
                }
            }
            
            println!("Decoded: {}", decoded);
            break;
        }
    }
}