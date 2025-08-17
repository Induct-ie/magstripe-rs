use super::common::{calculate_lrc_track1, check_parity, extract_bits, invert_bits};
use crate::{BitStream, DecoderError, ParityType};

const TRACK1_START_SENTINEL: u8 = 0b0000101; // '%' (0x25 - 0x20 = 0x05)
const TRACK1_END_SENTINEL: u8 = 0b0011111; // '?' (0x3F - 0x20 = 0x1F)

/// Decode Track 1 IATA format
pub fn decode_track1(stream: &BitStream, inverted: bool) -> Result<String, DecoderError> {
    // Track 1 uses 7-bit characters
    const BITS_PER_CHAR: u8 = 7;

    // Check minimum length
    if stream.len() < 21 {
        return Err(DecoderError::BitstreamTooShort {
            bit_count: stream.len(),
            minimum_required: 21,
        });
    }

    let mut result = String::new();
    let mut offset = 0;
    let mut found_start = false;
    let mut chars_read = Vec::new();

    // Process the stream
    while offset + BITS_PER_CHAR as usize <= stream.len() {
        // Extract character bits (LSB first)
        let mut char_bits =
            extract_bits(stream, offset, BITS_PER_CHAR).ok_or(DecoderError::BitstreamTooShort {
                bit_count: stream.len(),
                minimum_required: offset + BITS_PER_CHAR as usize,
            })?;

        // Apply inversion if needed
        if inverted {
            char_bits = invert_bits(char_bits) & 0x7F; // Keep only 7 bits
        }

        // Check parity (Track 1 uses odd parity on all 7 bits)
        if !check_parity(char_bits, 7, &ParityType::Odd) {
            return Err(DecoderError::ParityError {
                position: offset / BITS_PER_CHAR as usize,
            });
        }

        // Extract the 6 data bits (bits 0-5)
        let data_bits = char_bits & 0x3F;

        // Store for LRC calculation
        chars_read.push(char_bits);

        // Check for start sentinel
        if !found_start {
            if data_bits == TRACK1_START_SENTINEL {
                found_start = true;
            }
            offset += BITS_PER_CHAR as usize;
            continue;
        }

        // Check for end sentinel
        if data_bits == TRACK1_END_SENTINEL {
            // Read LRC character
            offset += BITS_PER_CHAR as usize;
            if offset + BITS_PER_CHAR as usize <= stream.len() {
                let lrc_bits = extract_bits(stream, offset, BITS_PER_CHAR).unwrap_or(0);

                // Verify LRC
                let calculated_lrc = calculate_lrc_track1(&chars_read[..chars_read.len() - 1]);
                if (lrc_bits & 0x7F) != calculated_lrc {
                    return Err(DecoderError::LrcCheckFailed);
                }
            }
            break;
        }

        // Decode the character
        let decoded_char = decode_track1_character(data_bits)?;
        result.push(decoded_char);

        offset += BITS_PER_CHAR as usize;
    }

    // Check if we found the start sentinel
    if !found_start {
        return Err(DecoderError::InvalidStartSentinel);
    }

    if result.is_empty() {
        return Err(DecoderError::NoValidFormat { attempted: 1 });
    }

    Ok(result)
}

/// Decode a single Track 1 character from 6 data bits
fn decode_track1_character(data_bits: u8) -> Result<char, DecoderError> {
    // Track 1 uses ASCII with offset of 32 (0x20)
    // Valid range is 0x20-0x5F in ASCII (space to underscore)
    let ascii_code = 0x20 + data_bits;

    // Check if it's a valid printable character
    if (0x20..=0x5F).contains(&ascii_code) {
        Ok(ascii_code as char)
    } else {
        Err(DecoderError::InvalidCharacter {
            position: 0,
            character: data_bits,
        })
    }
}
