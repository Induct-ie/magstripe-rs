/// Understand how the working decoder interprets the bits
#[test] 
fn understand_working_decoder_encoding() {
    // The working decoder shows these mappings:
    // 00001 -> 0
    // 00100 -> 4
    // 00010 -> 8
    // 11100 -> 7
    // 10000 -> 1
    // 01000 -> 2
    
    println!("Working decoder bit patterns:");
    let patterns = vec![
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
    
    for &(bits, expected_char) in &patterns {
        // Extract just the data bits (lower 4 bits)
        let data_bits = bits & 0x0F;
        
        // Standard Track 2 would do: 0x30 + data_bits
        let standard_char = (0x30 + data_bits) as char;
        
        // But maybe they're just using the data bits as the digit directly?
        let direct_digit = if data_bits <= 9 {
            (b'0' + data_bits) as char
        } else {
            '?'
        };
        
        // Or maybe there's bit reordering?
        let mut reversed = 0u8;
        for i in 0..4 {
            if (data_bits >> i) & 1 == 1 {
                reversed |= 1 << (3 - i);
            }
        }
        let reversed_char = (b'0' + reversed) as char;
        
        println!("Bits {:05b}: data={:04b} ({})", bits, data_bits, data_bits);
        println!("  Standard Track2: '{}' (expected '{}')", standard_char, expected_char);
        println!("  Direct digit: '{}' (expected '{}')", direct_digit, expected_char);
        println!("  Reversed: '{}' (expected '{}')", reversed_char, expected_char);
        
        // Check which one matches
        if direct_digit == expected_char {
            println!("  ✓ Direct digit matches!");
        } else if reversed_char == expected_char {
            println!("  ✓ Reversed bits match!");
        }
        println!();
    }
    
    // Aha! Let me check if the bits are being reversed
    println!("\nChecking bit reversal pattern:");
    println!("0b0001 reversed = 0b1000 = 8");
    println!("0b0100 reversed = 0b0010 = 2");
    println!("0b0010 reversed = 0b0100 = 4");
    println!("0b1100 reversed = 0b0011 = 3");
    println!("0b0000 reversed = 0b0000 = 0");
    println!("0b1000 reversed = 0b0001 = 1");
    
    // No wait, let me check the actual values:
    println!("\nActual check:");
    println!("00001 has data bits 0001, if we use them directly = 1, but expected 0");
    println!("00100 has data bits 0100, if we use them directly = 4, expected 4 ✓");
    println!("00010 has data bits 0010, if we use them directly = 2, but expected 8");
    
    // Hmm, it's not consistent. Let me check if there's a different bit extraction
    println!("\nMaybe they extract bits differently?");
    println!("If 00001 -> 0, then bit 0 is the data");
    println!("If 00100 -> 4, then bits 2 is set for value 4");
    println!("If 00010 -> 8, then bit 1 is set for value 8");
    println!("If 11100 -> 7, then bits 234 = 111 = 7");
    println!("If 10000 -> 1, then bit 4 is set for value 1");
    
    // Actually, let me check if they're reading the BITS in reverse order
    println!("\nChecking if bits are read in reverse:");
    for &(bits, expected_char) in &patterns {
        // Reverse the entire 5-bit value
        let mut reversed = 0u8;
        for i in 0..5 {
            if (bits >> i) & 1 == 1 {
                reversed |= 1 << (4 - i);
            }
        }
        
        // Now extract data bits
        let data_bits = reversed & 0x0F;
        let ch = (b'0' + data_bits) as char;
        
        println!("{:05b} -> reversed {:05b} -> data {:04b} -> '{}' (expected '{}')",
                 bits, reversed, data_bits, ch, expected_char);
        
        if ch == expected_char {
            println!("  ✓ MATCH!");
        }
    }
}