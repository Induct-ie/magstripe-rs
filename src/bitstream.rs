use std::fmt;

/// An immutable bit stream that wraps a byte slice with a specific bit count.
/// 
/// The internal buffer is left-aligned, meaning the significant bits start from
/// the MSB of the first byte, and any trailing bits in the last byte are zeroed.
/// The buffer is automatically shrunk to the minimum size needed to store the
/// specified number of bits during construction.
#[derive(Clone, PartialEq, Eq)]
pub struct BitStream<'a> {
    buffer: &'a [u8],
    bit_count: usize,
}

/// Errors that can occur when creating a BitStream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BitStreamError {
    /// The provided buffer is too small to hold the specified number of bits.
    BufferTooSmall {
        required_bytes: usize,
        provided_bytes: usize,
    },
}

impl fmt::Display for BitStreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BitStreamError::BufferTooSmall { required_bytes, provided_bytes } => {
                write!(
                    f,
                    "Buffer too small: {} bytes required, but only {} bytes provided",
                    required_bytes, provided_bytes
                )
            }
        }
    }
}

impl std::error::Error for BitStreamError {}

impl<'a> BitStream<'a> {
    /// Creates a new BitStream from a byte slice and bit count.
    /// 
    /// The buffer will be shrunk to the minimum size needed to hold the specified
    /// number of bits. The last bits in the final byte must be zeroed if they
    /// are not part of the bit stream.
    /// 
    /// # Arguments
    /// 
    /// * `buffer` - The byte slice containing the bit data
    /// * `bit_count` - The number of valid bits in the stream
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(BitStream)` if the buffer is large enough, or an error if the
    /// buffer is too small to hold the specified number of bits.
    /// 
    /// # Example
    /// 
    /// ```
    /// let data = vec![0b11010110, 0b10100000];
    /// let stream = magstripe_rs::BitStream::new(&data, 12).unwrap();
    /// assert_eq!(stream.len(), 12);
    /// ```
    pub fn new(buffer: &'a [u8], bit_count: usize) -> Result<Self, BitStreamError> {
        // Calculate the minimum number of bytes needed
        let required_bytes = bit_count.div_ceil(8);
        
        if buffer.len() < required_bytes {
            return Err(BitStreamError::BufferTooSmall {
                required_bytes,
                provided_bytes: buffer.len(),
            });
        }
        
        // Shrink the buffer to the minimum required size
        let shrunk_buffer = &buffer[..required_bytes];
        
        Ok(BitStream {
            buffer: shrunk_buffer,
            bit_count,
        })
    }
    
    /// Returns the number of bits in the stream.
    #[inline]
    pub fn len(&self) -> usize {
        self.bit_count
    }
    
    /// Returns true if the bit stream contains no bits.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bit_count == 0
    }
    
    /// Returns the internal byte buffer.
    /// 
    /// The buffer is left-aligned, with any trailing bits in the last byte zeroed.
    #[inline]
    pub fn buffer(&self) -> &'a [u8] {
        self.buffer
    }
}

impl<'a> fmt::Debug for BitStream<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BitStream(")?;
        
        if self.bit_count == 0 {
            write!(f, ")")?;
            return Ok(());
        }
        
        let mut bits_written = 0;
        
        for &byte in self.buffer.iter() {
            // Determine how many bits to show from this byte
            let bits_remaining = self.bit_count - bits_written;
            let bits_in_byte = bits_remaining.min(8);
            
            // Print each bit
            for bit_idx in 0..bits_in_byte {
                // Extract bit from MSB side (left-aligned)
                let bit = (byte >> (7 - bit_idx)) & 1;
                write!(f, "{bit}")?;
                bits_written += 1;
                
                // Add colon after every 8 bits, except at the end
                if bits_written % 8 == 0 && bits_written < self.bit_count {
                    write!(f, ":")?;
                }
            }
        }
        
        write!(f, ")")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_valid() {
        let data = vec![0xFF, 0x00, 0xAA];
        let stream = BitStream::new(&data, 20).unwrap();
        assert_eq!(stream.len(), 20);
        assert_eq!(stream.buffer().len(), 3);
    }
    
    #[test]
    fn test_new_shrinks_buffer() {
        let data = vec![0xFF, 0x00, 0xAA, 0xBB, 0xCC];
        let stream = BitStream::new(&data, 20).unwrap();
        assert_eq!(stream.buffer().len(), 3); // Should only keep 3 bytes
    }
    
    #[test]
    fn test_new_buffer_too_small() {
        let data = vec![0xFF];
        let result = BitStream::new(&data, 16);
        assert!(matches!(
            result,
            Err(BitStreamError::BufferTooSmall { 
                required_bytes: 2, 
                provided_bytes: 1 
            })
        ));
    }
    
    #[test]
    fn test_debug_format() {
        // Test with exactly 8 bits
        let data = vec![0b11010110];
        let stream = BitStream::new(&data, 8).unwrap();
        let debug_str = format!("{:?}", stream);
        assert_eq!(debug_str, "BitStream(11010110)");
        
        // Test with 12 bits (1.5 bytes)
        let data = vec![0b11010110, 0b10100000];
        let stream = BitStream::new(&data, 12).unwrap();
        let debug_str = format!("{:?}", stream);
        assert_eq!(debug_str, "BitStream(11010110:1010)");
        
        // Test with 16 bits (2 bytes)
        let data = vec![0b11010110, 0b10101111];
        let stream = BitStream::new(&data, 16).unwrap();
        let debug_str = format!("{:?}", stream);
        assert_eq!(debug_str, "BitStream(11010110:10101111)");
        
        // Test with 20 bits (2.5 bytes)
        let data = vec![0b11010110, 0b10101111, 0b11000000];
        let stream = BitStream::new(&data, 20).unwrap();
        let debug_str = format!("{:?}", stream);
        assert_eq!(debug_str, "BitStream(11010110:10101111:1100)");
    }
    
    #[test]
    fn test_empty_stream() {
        let data = vec![];
        let stream = BitStream::new(&data, 0).unwrap();
        assert_eq!(stream.len(), 0);
        assert!(stream.is_empty());
        let debug_str = format!("{:?}", stream);
        assert_eq!(debug_str, "BitStream()");
    }
}
