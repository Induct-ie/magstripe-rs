use super::common::{
    calculate_lrc_track2, check_parity, extract_bits,
};
use crate::{decoder::common::calculate_lrc_track1, BitStream, DecoderError, ParityType};
use tracing::{debug, trace};

const TRACK2_START_SENTINEL: u8 = 0b01011; // ';'
const TRACK2_END_SENTINEL:   u8 = 0b11111; // '?'

#[inline]
fn bitrev5(v: u8) -> u8 {
    // reverse the low 5 bits
    ((v & 0b00001) << 4) |
    ((v & 0b00010) << 2) |
    ( v & 0b00100)       |
    ((v & 0b01000) >> 2) |
    ((v & 0b10000) >> 4)
}

///
/// Read a 5-bit character from the stream
/// 
/// Returns the character bits if successful, None if the stream is too short
/// 
#[inline]
fn read_char5(stream: &BitStream, off: usize, lsb_first_on_wire: bool, inverted: bool) -> Option<u8> {
    // Always grab with the LSB-accumulating extractor
    let mut v = extract_bits(stream, off, 5)?;
    // If wire is MSB-first, reverse to canonical dddd p
    if !lsb_first_on_wire {
        v = bitrev5(v);
    }
    if inverted {
        v ^= 0x1F;
    }
    Some(v & 0x1F) // canonical: data in bits 0..3, parity in bit 4
}



/// Decode Track 2 format with various options
pub fn decode_track2(
    stream: &BitStream,
    inverted: bool,
    lsb_first: bool,
    no_sentinels: bool,
    swapped_parity: bool,
    even_parity: bool,
) -> Result<String, DecoderError> {
    debug!("Decoding Track 2 with inverted: {}, lsb_first: {}, no_sentinels: {}, swapped_parity: {}, even_parity: {}", inverted, lsb_first, no_sentinels, swapped_parity, even_parity);
    
    // Track 2 uses 5-bit characters
    const BITS_PER_CHAR: u8 = 5;

    // Check minimum length (at least start + end sentinels + 1 char)
    if !no_sentinels && stream.len() < 15 {
        return Err(DecoderError::BitstreamTooShort {
            bit_count: stream.len(),
            minimum_required: 15,
        });
    }

    let mut result = String::new();
    let mut offset = 0;
    let mut found_start = no_sentinels; // Skip start check if no sentinels
    let mut chars_read = Vec::new();

    // First, search for start sentinel with single-bit alignment if needed
    if !found_start {
        let mut search_offset = 0;
        while search_offset + BITS_PER_CHAR as usize <= stream.len() {

            
            // Extract character bits
            let char_bits = read_char5(stream, search_offset, lsb_first, inverted)
            .ok_or(DecoderError::BitstreamTooShort {
                bit_count: stream.len(),
                minimum_required: search_offset + BITS_PER_CHAR as usize,
            })?;

            // Check for start sentinel
            if char_bits == TRACK2_START_SENTINEL {
                debug!(
                    "Found start sentinel {:05b} at bit offset {}",
                    char_bits, search_offset
                );
                found_start = true;
                chars_read.push(char_bits);
                offset = search_offset + BITS_PER_CHAR as usize;
                break;
            }
            
            search_offset += 1; // Check every single bit position
        }
        
        if !found_start {
            return Err(DecoderError::InvalidStartSentinel);
        }
    }

    // Debug: Print the remaining stream length and data
    debug!("Remaining stream length: {}", stream.len() - offset);
    debug!("Remaining stream data: {:?}", stream.buffer()[offset/8..].to_vec());

    // Process the stream - now that we found the start sentinel
    while offset <= stream.len() - BITS_PER_CHAR as usize {
        // Debug: Print the offset
        debug!("Offset: {}", offset);

        // Extract character bits
        let char_bits = read_char5(stream, offset, lsb_first, inverted)
        .ok_or(DecoderError::BitstreamTooShort {
            bit_count: stream.len(),
            minimum_required: offset + BITS_PER_CHAR as usize,
        })?;
    
        

        // Debug: Print the character bits
        debug!("Character bits: {:05b}", char_bits);
        debug!("Remaining stream length: {}", stream.len() - offset);

        // Extract data and parity bits
        let (data_bits, _parity_bit) = if swapped_parity {
            // Parity in different position (implementation specific)
            (char_bits & 0x0F, (char_bits >> 4) & 1)
        } else {
            // Standard: 4 data bits + 1 parity bit
            (char_bits & 0x0F, (char_bits >> 4) & 1)
        };

        // Check parity (Track 2 normally uses odd parity, but some cards use even)
        let parity_type = if even_parity {
            ParityType::Even
        } else {
            ParityType::Odd
        };
        if !check_parity(char_bits, 5, &parity_type) {
            return Err(DecoderError::ParityError {
                position: offset / BITS_PER_CHAR as usize,
            });
        }

        // Store the full character for LRC calculation
        chars_read.push(char_bits);

        // Check for end sentinel
        if !no_sentinels && char_bits == TRACK2_END_SENTINEL {
            debug!("Found end sentinel at bit offset {}", offset);
            // Read LRC character
            offset += BITS_PER_CHAR as usize;
            if offset + BITS_PER_CHAR as usize <= stream.len() {
                let lrc_bits = read_char5(stream, offset, lsb_first, inverted)
                .ok_or(DecoderError::BitstreamTooShort {
                    bit_count: stream.len(),
                    minimum_required: offset + BITS_PER_CHAR as usize,
                })?;

                // Verify LRC
                // If the line is inverted, we need to invert the LRC bits to match the parity
                let mut calculated_lrc = calculate_lrc_track2(&chars_read[..chars_read.len() - 1]);
                if inverted {
                    calculated_lrc ^= 0x1F;
                }

                debug!("Calculated LRC: {:05b}", calculated_lrc);
                debug!("LRC bits: {:05b}", lrc_bits);
                if lrc_bits != calculated_lrc {
                    return Err(DecoderError::LrcCheckFailed);
                }
            }
            break;
        }

        // Decode the character
        let decoded_char = decode_track2_character(data_bits)?;
        debug!("Decoded character: {}", decoded_char);
        result.push(decoded_char);

        offset += BITS_PER_CHAR as usize;
    }


    if result.is_empty() {
        return Err(DecoderError::NoValidFormat { attempted: 1 });
    }

    debug!("Track2 decoded successfully: {} characters", result.len());
    trace!("Decoded data: {}", result);
    Ok(result)
}

/// Decode a single Track 2 character from 4 data bits
fn decode_track2_character(data_bits: u8) -> Result<char, DecoderError> {
    // Track 2 character set: 0-9, :, ;, <, =, >, ?
    // Data bits 0-15 map to ASCII 0x30-0x3F
    let ascii_code = 0x30 + data_bits;

    match ascii_code {
        0x30..=0x39 => Ok(ascii_code as char), // 0-9
        0x3A => Ok(':'),
        0x3B => Ok(';'),
        0x3C => Ok('<'),
        0x3D => Ok('='),
        0x3E => Ok('>'),
        0x3F => Ok('?'),
        _ => Err(DecoderError::InvalidCharacter {
            position: 0,
            character: data_bits,
        }),
    }
}
