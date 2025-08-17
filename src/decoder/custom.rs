use super::common::{check_parity, extract_bits, extract_bits_msb, invert_bits};
use crate::{BitStream, DecoderError, FormatSpec, ParityType};

/// Decode using a custom format specification
pub fn decode_custom(stream: &BitStream, spec: &FormatSpec) -> Result<String, DecoderError> {
    // Validate the format specification
    if spec.bits_per_char == 0 || spec.bits_per_char > 8 {
        return Err(DecoderError::InvalidCustomFormat {
            reason: format!("Invalid bits_per_char: {}", spec.bits_per_char),
        });
    }

    let mut result = String::new();
    let mut offset = 0;
    let mut found_start = spec.start_sentinel.is_none();
    let mut found_end = false;

    // Process the stream
    while offset + spec.bits_per_char as usize <= stream.len() && !found_end {
        // Extract character bits
        let mut char_bits = if spec.lsb_first {
            extract_bits(stream, offset, spec.bits_per_char)
        } else {
            extract_bits_msb(stream, offset, spec.bits_per_char)
        }
        .ok_or(DecoderError::BitstreamTooShort {
            bit_count: stream.len(),
            minimum_required: offset + spec.bits_per_char as usize,
        })?;

        // Apply inversion if needed
        if spec.inverted {
            let mask = (1u8 << spec.bits_per_char) - 1;
            char_bits = invert_bits(char_bits) & mask;
        }

        // Check parity if required
        if spec.parity != ParityType::None {
            if !check_parity(char_bits, spec.bits_per_char, &spec.parity) {
                return Err(DecoderError::ParityError {
                    position: offset / spec.bits_per_char as usize,
                });
            }
        }

        // Check for start sentinel
        if let Some(start_sentinel) = spec.start_sentinel {
            if !found_start {
                if char_bits == start_sentinel {
                    found_start = true;
                }
                offset += spec.bits_per_char as usize;
                continue;
            }
        }

        // Check for end sentinel
        if let Some(end_sentinel) = spec.end_sentinel {
            if char_bits == end_sentinel {
                found_end = true;
                break;
            }
        }

        // Decode the character based on bits per character
        let decoded_char = decode_custom_character(char_bits, spec)?;
        result.push(decoded_char);

        offset += spec.bits_per_char as usize;
    }

    // Check if we found required sentinels
    if spec.start_sentinel.is_some() && !found_start {
        return Err(DecoderError::InvalidStartSentinel);
    }

    if spec.end_sentinel.is_some() && !found_end {
        return Err(DecoderError::InvalidEndSentinel);
    }

    if result.is_empty() {
        return Err(DecoderError::NoValidFormat { attempted: 1 });
    }

    Ok(result)
}

/// Decode a character for custom format
fn decode_custom_character(char_bits: u8, spec: &FormatSpec) -> Result<char, DecoderError> {
    // Remove parity bit if present
    let data_bits = if spec.parity != ParityType::None && spec.bits_per_char > 1 {
        // Assume parity is the highest bit
        let data_mask = (1u8 << (spec.bits_per_char - 1)) - 1;
        char_bits & data_mask
    } else {
        char_bits
    };

    // Decode based on the number of bits
    match spec.bits_per_char {
        5 => {
            // Track 2 style encoding
            let ascii_code = 0x30 + (data_bits & 0x0F);
            match ascii_code {
                0x30..=0x3F => Ok(ascii_code as char),
                _ => Err(DecoderError::InvalidCharacter {
                    position: 0,
                    character: data_bits,
                }),
            }
        }
        7 => {
            // Track 1 style encoding
            let ascii_code = 0x20 + (data_bits & 0x3F);
            if ascii_code >= 0x20 && ascii_code <= 0x5F {
                Ok(ascii_code as char)
            } else {
                Err(DecoderError::InvalidCharacter {
                    position: 0,
                    character: data_bits,
                })
            }
        }
        8 => {
            // Direct ASCII
            Ok(data_bits as char)
        }
        _ => {
            // For other bit sizes, try to interpret as numeric
            if data_bits <= 9 {
                Ok((b'0' + data_bits) as char)
            } else {
                Err(DecoderError::InvalidCharacter {
                    position: 0,
                    character: data_bits,
                })
            }
        }
    }
}
