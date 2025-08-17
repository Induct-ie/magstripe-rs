/// Detailed bit-level analysis of the test card
#[test]
fn analyze_test_card_bits() {
    let data = vec![255, 255, 255, 151, 222, 246, 253, 190, 141, 247, 7, 127, 255, 255, 255, 255, 192];
    
    println!("Analyzing card data bit by bit:");
    println!("================================");
    
    // Print all bits
    println!("\nAll bits:");
    for (byte_idx, &byte) in data.iter().enumerate() {
        for bit_idx in 0..8 {
            let bit = (byte >> (7 - bit_idx)) & 1;
            print!("{}", bit);
        }
        if byte_idx < data.len() - 1 {
            print!(" ");
        }
    }
    println!("\n");
    
    // Look for Track2 start sentinel (11010) at offset 26
    println!("Extracting 5-bit characters starting at bit offset 26:");
    
    let mut offset = 26;
    let mut char_count = 0;
    
    while offset + 5 <= 130 && char_count < 30 {
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
        
        // Check parity
        let mut ones = 0;
        for i in 0..5 {
            if (value >> i) & 1 == 1 {
                ones += 1;
            }
        }
        let odd_parity = ones % 2 == 1;
        let even_parity = ones % 2 == 0;
        
        // Decode character
        let data_bits = value & 0x0F;
        let ch = (0x30 + data_bits) as char;
        
        println!("  Offset {:3}: {:05b} => data={:04b} ({}) '{}' - odd_parity={}, even_parity={}",
                 offset, value, data_bits, data_bits, ch, odd_parity, even_parity);
        
        // Check for sentinels
        if value == 0b11010 {
            println!("    ^ START SENTINEL (;)");
        } else if value == 0b11111 {
            println!("    ^ END SENTINEL (?)");
        } else if value == 0b11101 {
            println!("    ^ FIELD SEPARATOR (=)");
        }
        
        offset += 5;
        char_count += 1;
    }
    
    // Now let's check what happens if we treat the parity as even
    println!("\n\nChecking with EVEN parity instead of odd:");
    offset = 26;
    let mut decoded = String::new();
    let mut errors = 0;
    
    while offset + 5 <= 130 {
        let mut value = 0u8;
        
        for bit_idx in 0..5 {
            let absolute_bit = offset + bit_idx;
            let byte_idx = absolute_bit / 8;
            let bit_in_byte = absolute_bit % 8;
            
            if byte_idx < data.len() {
                let bit = (data[byte_idx] >> (7 - bit_in_byte)) & 1;
                value |= bit << bit_idx;
            }
        }
        
        // Check for end sentinel
        if value == 0b11111 {
            println!("Found END at offset {}", offset);
            break;
        }
        
        // Skip start sentinel
        if value == 0b11010 {
            offset += 5;
            continue;
        }
        
        // Check even parity
        let mut ones = 0;
        for i in 0..5 {
            if (value >> i) & 1 == 1 {
                ones += 1;
            }
        }
        
        if ones % 2 != 0 {  // Even parity check
            errors += 1;
        }
        
        let data_bits = value & 0x0F;
        let ch = (0x30 + data_bits) as char;
        decoded.push(ch);
        
        offset += 5;
    }
    
    println!("Decoded with even parity: {}", decoded);
    println!("Parity errors: {}", errors);
}