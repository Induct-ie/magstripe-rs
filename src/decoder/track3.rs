use crate::{BitStream, DecoderError};
use super::track2::decode_track2;

/// Decode Track 3 format
/// Track 3 uses the same encoding as Track 2 (5-bit) but at higher density (210 bpi vs 75 bpi)
pub fn decode_track3(stream: &BitStream) -> Result<String, DecoderError> {
    // Track 3 uses the same encoding scheme as Track 2
    // The only difference is the recording density, which doesn't affect decoding logic
    decode_track2(stream, false, true, false, false, false)
}