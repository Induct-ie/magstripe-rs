use crate::{BitStream, DecoderError, ParityType};
use super::common::{extract_bits, extract_bits_msb, invert_bits, check_parity, calculate_lrc_track2};
use tracing::{debug, trace};

const TRACK2_START_SENTINEL: u8 = 0b11010; // ';' 
const TRACK2_START_SENTINEL_ALT: u8 = 0b01011; // Alternative pattern sometimes seen
const TRACK2_END_SENTINEL: u8 = 0b11111;   // '?'

/// Decode Track 2 format with various options
pub fn decode_track2(
    stream: &BitStream,
    inverted: bool,
    lsb_first: bool,
    no_sentinels: bool,
    swapped_parity: bool,
    even_parity: bool,
) -> Result<String, DecoderError> {
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
    
    // Process the stream - search for start sentinel or process data
    while offset + BITS_PER_CHAR as usize <= stream.len() {
        // Extract character bits
        let mut char_bits = if lsb_first {
            extract_bits(stream, offset, BITS_PER_CHAR)
        } else {
            extract_bits_msb(stream, offset, BITS_PER_CHAR)
        }.ok_or(DecoderError::BitstreamTooShort {
            bit_count: stream.len(),
            minimum_required: offset + BITS_PER_CHAR as usize,
        })?;
        
        // Apply inversion if needed
        if inverted {
            char_bits = invert_bits(char_bits) & 0x1F; // Keep only 5 bits
        }
        
        // Check for start sentinel before checking parity
        if !found_start {
            if char_bits == TRACK2_START_SENTINEL || char_bits == TRACK2_START_SENTINEL_ALT {
                debug!("Found start sentinel {:05b} at bit offset {}", char_bits, offset);
                found_start = true;
                chars_read.push(char_bits);
            }
            offset += BITS_PER_CHAR as usize;
            continue;
        }
        
        // Extract data and parity bits
        let (data_bits, _parity_bit) = if swapped_parity {
            // Parity in different position (implementation specific)
            (char_bits & 0x0F, (char_bits >> 4) & 1)
        } else {
            // Standard: 4 data bits + 1 parity bit
            (char_bits & 0x0F, (char_bits >> 4) & 1)
        };
        
        // Check parity (Track 2 normally uses odd parity, but some cards use even)
        let parity_type = if even_parity { ParityType::Even } else { ParityType::Odd };
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
                let lrc_bits = if lsb_first {
                    extract_bits(stream, offset, BITS_PER_CHAR)
                } else {
                    extract_bits_msb(stream, offset, BITS_PER_CHAR)
                }.unwrap_or(0);
                
                // Verify LRC
                let calculated_lrc = calculate_lrc_track2(&chars_read[..chars_read.len() - 1]);
                if lrc_bits != calculated_lrc {
                    return Err(DecoderError::LrcCheckFailed);
                }
            }
            break;
        }
        
        // Decode the character
        let decoded_char = decode_track2_character(data_bits)?;
        result.push(decoded_char);
        
        offset += BITS_PER_CHAR as usize;
    }
    
    // Check if we found sentinels when required
    if !no_sentinels && !found_start {
        return Err(DecoderError::InvalidStartSentinel);
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
