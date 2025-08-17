/// Find the exact format that works for this card
#[test]
fn find_correct_format() {
    let data = vec![
        255, 255, 255, 151, 222, 246, 253, 190, 141, 247, 7, 127, 255, 255, 255, 255, 192,
    ];

    // The working decoder shows:
    // - Inverted data
    // - Start sentinel at bit 25
    // - Decodes to "0004048712"

    let inverted: Vec<u8> = data.iter().map(|&b| !b).collect();

    println!("Testing different bit extraction methods at offset 25:");

    // Try LSB first (this is what Track2Inverted actually uses)
    println!("\nLSB-first extraction:");
    let mut value_lsb = 0u8;
    for bit_idx in 0..5 {
        let absolute_bit = 25 + bit_idx;
        let byte_idx = absolute_bit / 8;
        let bit_in_byte = absolute_bit % 8;
        let bit = (inverted[byte_idx] >> (7 - bit_in_byte)) & 1;
        value_lsb |= bit << bit_idx;
        println!("  Bit {}: {}", absolute_bit, bit);
    }
    println!(
        "  Result: {:05b} = {} ({})",
        value_lsb,
        value_lsb,
        if value_lsb == 0b11010 || value_lsb == 0b01011 {
            "START!"
        } else {
            "not start"
        }
    );

    // The decoder actually uses LSB-first extraction
    // The alternate start sentinel 0b01011 is also valid
    if value_lsb == 0b01011 {
        println!("\n✓ LSB-first gives us the alternate start sentinel (0b01011)!");

        // Now decode the rest using LSB-first
        println!("\nDecoding with LSB-first:");
        let mut offset = 30; // After start sentinel
        let mut decoded = String::new();

        for i in 0..10 {
            let mut char_val = 0u8;
            // Extract bits LSB-first
            for bit_idx in 0..5 {
                let absolute_bit = offset + bit_idx;
                let byte_idx = absolute_bit / 8;
                let bit_in_byte = absolute_bit % 8;
                let bit = (inverted[byte_idx] >> (7 - bit_in_byte)) & 1;
                char_val |= bit << bit_idx;
            }

            let data_bits = char_val & 0x0F;
            let ch = (0x30 + data_bits) as char;

            // Check parity
            let mut ones = 0;
            for j in 0..5 {
                if (char_val >> j) & 1 == 1 {
                    ones += 1;
                }
            }
            let parity_ok = ones % 2 == 1;

            println!(
                "  Char {}: {:05b} => '{}' {}",
                i,
                char_val,
                ch,
                if parity_ok { "✓" } else { "✗" }
            );

            decoded.push(ch);
            offset += 5;
        }

        println!("\nDecoded: {}", decoded);
        assert_eq!(decoded, "0004048712");
    }
}

