mod common;
mod custom;
mod track1;
mod track2;
mod track3;

use crate::{BitStream, DecoderError, DecoderOutput, Format};
use tracing::{debug, trace, warn};

/// Main decode implementation that tries each format
pub fn decode_with_formats<'a>(
    formats: &'a [Format],
    stream: BitStream,
) -> Result<DecoderOutput<'a>, DecoderError> {
    // Check if any formats were provided
    if formats.is_empty() {
        warn!("No formats provided for decoding");
        return Err(DecoderError::NoFormatsProvided);
    }

    debug!(
        "Starting decode with {} formats, bitstream length: {} bits",
        formats.len(),
        stream.len()
    );
    trace!("Bitstream: {:?}", stream);

    // Try each format in order
    for format in formats {
        debug!("Trying format: {:?}", format);
        match try_decode_format(format, &stream) {
            Ok(data) => {
                debug!("Successfully decoded with {:?}: {}", format, data);
                return Ok(DecoderOutput { data, format });
            }
            Err(e) => {
                trace!("Format {:?} failed: {:?}", format, e);
                // Continue to next format
                continue;
            }
        }
    }

    // None of the formats worked
    warn!("Failed to decode with any of {} formats", formats.len());
    Err(DecoderError::NoValidFormat {
        attempted: formats.len(),
    })
}

/// Try to decode with a specific format
fn try_decode_format(format: &Format, stream: &BitStream) -> Result<String, DecoderError> {
    match format {
        Format::Track2 => track2::decode_track2(stream, false, true, false, false, false),
        Format::Track2Inverted => track2::decode_track2(stream, true, true, false, false, false),
        Format::Track2MSB => track2::decode_track2(stream, false, false, false, false, false),
        Format::Track2LSB => track2::decode_track2(stream, false, true, false, false, false),
        Format::Track2Raw => track2::decode_track2(stream, false, true, true, false, false),
        Format::Track2SwappedParity => {
            track2::decode_track2(stream, false, true, false, true, false)
        }
        Format::Track2EvenParity => track2::decode_track2(stream, false, true, false, false, true),

        Format::Track1 => track1::decode_track1(stream, false),
        Format::Track1Inverted => track1::decode_track1(stream, true),

        Format::Track3 => track3::decode_track3(stream),

        Format::Custom(spec) => custom::decode_custom(stream, spec),
    }
}
