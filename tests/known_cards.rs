use magstripe_rs::{BitStream, Decoder, Format};

#[test]
fn test_card_1() {
    // Test data: 255, 255, 255, 151, 222, 246, 253, 190, 141, 247, 7, 127, 255, 255, 255, 255, 192
    // Length: 130 bits
    let data = vec![255, 255, 255, 151, 222, 246, 253, 190, 141, 247, 7, 127, 255, 255, 255, 255, 192];
    let stream = BitStream::new(&data, 130).unwrap();
    
    // Let's examine the bits to understand the format
    println!("BitStream: {:?}", stream);
    
    // Try various formats
    let formats = vec![
        Format::Track2,
        Format::Track2Inverted,
        Format::Track2MSB,
        Format::Track2LSB,
        Format::Track1,
        Format::Track1Inverted,
    ];
    
    let decoder = Decoder::new(&formats);
    
    match decoder.decode(stream.clone()) {
        Ok(output) => {
            println!("Successfully decoded with format: {:?}", output.format);
            println!("Data: {}", output.data);
            assert!(!output.data.is_empty());
        }
        Err(e) => {
            println!("Failed to decode: {:?}", e);
            
            // Let's try to understand the data by looking at the bits
            // The leading 255s suggest this might be inverted (all 1s)
            // Let's manually check what this looks like
            println!("\nAnalyzing bit pattern:");
            println!("First few bytes in binary:");
            for (i, byte) in data.iter().take(5).enumerate() {
                println!("Byte {}: {:08b}", i, byte);
            }
            
            panic!("Could not decode test card");
        }
    }
}

#[test] 
fn test_card_1_inverted_analysis() {
    // The card data appears to be inverted (lots of 255s = all 1s)
    // Let's invert it and see what we get
    let data = vec![255, 255, 255, 151, 222, 246, 253, 190, 141, 247, 7, 127, 255, 255, 255, 255, 192];
    
    // Invert all the bits
    let inverted_data: Vec<u8> = data.iter().map(|&b| !b).collect();
    
    println!("Inverted data:");
    for (i, byte) in inverted_data.iter().take(8).enumerate() {
        println!("Byte {}: {:3} = {:08b}", i, byte, byte);
    }
    
    let stream = BitStream::new(&inverted_data, 130).unwrap();
    println!("Inverted BitStream: {:?}", stream);
    
    // Try decoding the inverted version
    let formats = vec![
        Format::Track2,
        Format::Track2LSB,
        Format::Track1,
    ];
    
    let decoder = Decoder::new(&formats);
    
    match decoder.decode(stream) {
        Ok(output) => {
            println!("Successfully decoded inverted data with format: {:?}", output.format);
            println!("Data: {}", output.data);
        }
        Err(e) => {
            println!("Failed to decode inverted data: {:?}", e);
        }
    }
}