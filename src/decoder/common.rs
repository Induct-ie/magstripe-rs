use crate::{BitStream, ParityType};

/// Extract a single character's worth of bits from the stream
pub fn extract_bits(stream: &BitStream, offset: usize, bits_per_char: u8) -> Option<u8> {
    if offset + bits_per_char as usize > stream.len() {
        return None;
    }

    let buffer = stream.buffer();
    let mut result = 0u8;

    for bit_idx in 0..bits_per_char {
        let absolute_bit = offset + bit_idx as usize;
        let byte_idx = absolute_bit / 8;
        let bit_in_byte = absolute_bit % 8;

        if byte_idx >= buffer.len() {
            return None;
        }

        let bit = (buffer[byte_idx] >> (7 - bit_in_byte)) & 1;
        result |= bit << bit_idx;
    }

    Some(result)
}

/// Extract bits with MSB-first ordering
pub fn extract_bits_msb(stream: &BitStream, offset: usize, bits_per_char: u8) -> Option<u8> {
    if offset + bits_per_char as usize > stream.len() {
        return None;
    }

    let buffer = stream.buffer();
    let mut result = 0u8;

    for bit_idx in 0..bits_per_char {
        let absolute_bit = offset + bit_idx as usize;
        let byte_idx = absolute_bit / 8;
        let bit_in_byte = absolute_bit % 8;

        if byte_idx >= buffer.len() {
            return None;
        }

        let bit = (buffer[byte_idx] >> (7 - bit_in_byte)) & 1;
        result |= bit << (bits_per_char - 1 - bit_idx);
    }

    Some(result)
}

/// Invert all bits in a byte
pub fn invert_bits(byte: u8) -> u8 {
    !byte
}

/// Check parity of a value
pub fn check_parity(value: u8, bits: u8, parity_type: &ParityType) -> bool {
    match parity_type {
        ParityType::None => true,
        ParityType::Odd => {
            let mut count = 0;
            for i in 0..bits {
                if (value >> i) & 1 == 1 {
                    count += 1;
                }
            }
            count % 2 == 1
        }
        ParityType::Even => {
            let mut count = 0;
            for i in 0..bits {
                if (value >> i) & 1 == 1 {
                    count += 1;
                }
            }
            count % 2 == 0
        }
    }
}

/// Calculate LRC (Longitudinal Redundancy Check) for Track 2
pub fn calculate_lrc_track2(data: &[u8]) -> u8 {
    let mut lrc = 0u8;

    for &byte in data {
        // XOR all bytes together
        lrc ^= byte;
    }

    // For Track 2, LRC uses even parity (opposite of character parity)
    // Count the bits and adjust if needed
    let mut count = 0;
    for i in 0..5 {
        if (lrc >> i) & 1 == 1 {
            count += 1;
        }
    }

    // If odd number of bits, flip the parity bit to make it even
    if count % 2 == 1 {
        lrc ^= 0b10000; // Flip the 5th bit (parity bit)
    }

    lrc
}

/// Calculate LRC for Track 1
pub fn calculate_lrc_track1(data: &[u8]) -> u8 {
    let mut lrc = 0u8;

    for &byte in data {
        // XOR all bytes together (excluding parity bit)
        lrc ^= byte & 0x3F; // Only use the 6 data bits
    }

    // Add even parity to the 6 data bits
    let mut count = 0;
    for i in 0..6 {
        if (lrc >> i) & 1 == 1 {
            count += 1;
        }
    }

    // Set the 7th bit for odd parity of the first 6 bits
    if count % 2 == 0 {
        lrc |= 0x40; // Set bit 6
    }

    lrc
}
