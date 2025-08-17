#![allow(clippy::uninlined_format_args)]

use magstripe_rs::{BitStream, Decoder, Format};

/// Test the user's actual card data that should decode as Track2Inverted
#[test]
fn test_real_card_track2_inverted() {
    let data = vec![
        255, 255, 255, 151, 222, 246, 253, 190, 141, 247, 7, 127, 255, 255, 255, 255, 192,
    ];
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

/// Test automatic format detection with multiple formats
#[test]
fn test_format_auto_detection() {
    let data = vec![
        255, 255, 255, 151, 222, 246, 253, 190, 141, 247, 7, 127, 255, 255, 255, 255, 192,
    ];
    let stream = BitStream::new(&data, 130).unwrap();

    // Try multiple formats - should pick Track2Inverted
    let formats = vec![
        Format::Track1,
        Format::Track2,
        Format::Track2Inverted, // This should match
        Format::Track3,
    ];

    let decoder = Decoder::new(&formats);
    let output = decoder.decode(stream).unwrap();

    assert_eq!(output.data, "0004048712");
    assert!(matches!(output.format, Format::Track2Inverted));
}

/// Test that decoder fails gracefully with no formats
#[test]
fn test_no_formats_provided() {
    let data = vec![255, 255, 255];
    let stream = BitStream::new(&data, 24).unwrap();

    let decoder = Decoder::new(&[]);

    match decoder.decode(stream) {
        Err(magstripe_rs::DecoderError::NoFormatsProvided) => {
            // Expected error
        }
        _ => panic!("Should fail with NoFormatsProvided error"),
    }
}

/// Test that decoder handles short bitstreams correctly
#[test]
fn test_bitstream_too_short() {
    let data = vec![255];
    let stream = BitStream::new(&data, 5).unwrap();

    let decoder = Decoder::new(&[Format::Track2]);

    match decoder.decode(stream) {
        Err(magstripe_rs::DecoderError::BitstreamTooShort { .. }) => {
            // Expected error
        }
        Err(magstripe_rs::DecoderError::NoValidFormat { .. }) => {
            // Also acceptable - no format could decode it
        }
        _ => panic!("Should fail with BitstreamTooShort or NoValidFormat error"),
    }
}

/// Test decoding with all zeros (should fail)
#[test]
fn test_all_zeros() {
    let data = vec![0, 0, 0, 0, 0, 0];
    let stream = BitStream::new(&data, 48).unwrap();

    let decoder = Decoder::new(&[Format::Track2, Format::Track2Inverted]);

    match decoder.decode(stream) {
        Err(magstripe_rs::DecoderError::NoValidFormat { attempted }) => {
            assert_eq!(attempted, 2);
        }
        _ => panic!("Should fail to decode all zeros"),
    }
}

/// Test decoding with all ones (should fail)
#[test]
fn test_all_ones() {
    let data = vec![255, 255, 255, 255, 255, 255];
    let stream = BitStream::new(&data, 48).unwrap();

    let decoder = Decoder::new(&[Format::Track2, Format::Track2Inverted]);

    match decoder.decode(stream) {
        Err(magstripe_rs::DecoderError::NoValidFormat { attempted }) => {
            assert_eq!(attempted, 2);
        }
        _ => panic!("Should fail to decode all ones"),
    }
}
