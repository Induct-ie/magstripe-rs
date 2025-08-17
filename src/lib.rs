#![doc = include_str!("../README.md")]

mod bitstream;
mod decoder;

pub use bitstream::{BitStream, BitStreamError};

/// Represents the various encoding formats used for magnetic stripe cards.
///
/// Magnetic stripe cards contain up to three tracks of data, each with different
/// encoding standards developed by various industries. This enum provides support
/// for standard formats as well as common variations found in the wild.
///
/// # Standards
///
/// - **Track 1**: ISO/IEC 7811, developed by IATA (International Air Transport Association)
/// - **Track 2**: ISO/IEC 7813, developed by ABA (American Bankers Association)  
/// - **Track 3**: ISO/IEC 4909, developed by the thrift industry
#[derive(Debug, Clone, PartialEq)]
pub enum Format {
    /// Standard ABA Track 2 format (ISO/IEC 7813).
    ///
    /// The most common format for financial cards. Uses 5-bit encoding
    /// (4 data bits + 1 odd parity bit) at 75 bpi density. Characters
    /// are encoded LSB-first with the character set including digits 0-9
    /// and symbols `:;<=>?`. Data starts with `;` (start sentinel) and
    /// ends with `?` (end sentinel), followed by an LRC check character.
    /// Maximum capacity: 40 characters including sentinels.
    Track2,

    /// Track 2 format with all bits inverted.
    ///
    /// Some readers or cards may invert the magnetic polarity, resulting
    /// in all bits being flipped (0→1, 1→0). This variant handles such
    /// inversions while maintaining the same character encoding scheme.
    Track2Inverted,

    /// Track 2 format with MSB-first bit ordering.
    ///
    /// While standard Track 2 uses LSB-first bit order (e.g., decimal 4
    /// is encoded as `00100`), some implementations use MSB-first ordering
    /// (decimal 4 would be `00100` read in reverse). The parity bit
    /// position remains unchanged.
    Track2MSB,

    /// Explicit LSB-first Track 2 format.
    ///
    /// Functionally identical to `Track2` but explicitly specifies LSB-first
    /// bit ordering. Useful when disambiguating between different bit order
    /// implementations or when the standard format needs to be explicit.
    Track2LSB,

    /// Track 2 format without sentinel characters.
    ///
    /// Some proprietary systems omit the standard start (`;`) and end (`?`)
    /// sentinels, encoding only the raw data. This format processes the
    /// entire magnetic data as payload without looking for framing characters.
    Track2Raw,

    /// Track 2 format with parity bit in a different position.
    ///
    /// Standard Track 2 places the parity bit as the 5th bit (bit 4 when
    /// 0-indexed). This variant handles cards where the parity bit is
    /// placed in a different position within the 5-bit character, which
    /// can occur due to encoding errors or non-standard implementations.
    Track2SwappedParity,
    
    /// Track 2 format with even parity instead of odd.
    ///
    /// While standard Track 2 uses odd parity, some non-standard
    /// implementations use even parity. This variant processes Track 2
    /// data with even parity checking.
    Track2EvenParity,

    /// Standard IATA Track 1 format (ISO/IEC 7811).
    ///
    /// Developed for the airline industry, uses 7-bit encoding (6 data bits
    /// + 1 odd parity bit) at 210 bpi density. Supports 64 alphanumeric
    /// characters including A-Z, 0-9, and special symbols. Data starts
    /// with `%` (start sentinel) and ends with `?` (end sentinel), followed
    /// by an LRC. Characters are encoded LSB-first with ASCII offset of 32.
    /// Maximum capacity: 79 characters including sentinels.
    Track1,

    /// Track 1 format with all bits inverted.
    ///
    /// Handles Track 1 data where magnetic polarity is inverted, similar
    /// to `Track2Inverted`. Maintains the 7-bit IATA encoding scheme while
    /// flipping all bit values.
    Track1Inverted,

    /// Standard Track 3 format (ISO/IEC 4909).
    ///
    /// Rarely used track originally designed for the thrift industry with
    /// read/write capability. Uses 5-bit encoding like Track 2 but at
    /// 210 bpi density (same as Track 1). Can store up to 107 numeric
    /// characters. Primarily used in some European countries (notably
    /// Germany) for additional authorization data, PINs, or account limits.
    Track3,

    /// Custom format with user-defined specifications.
    ///
    /// Allows defining non-standard formats by specifying encoding
    /// parameters directly. Useful for proprietary card systems, legacy
    /// formats, or experimental implementations that don't conform to
    /// ISO standards.
    Custom(FormatSpec),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FormatSpec {
    pub bits_per_char: u8,
    pub start_sentinel: Option<u8>,
    pub end_sentinel: Option<u8>,
    pub lsb_first: bool,
    pub parity: ParityType,
    pub inverted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParityType {
    Odd,
    Even,
    None,
}

pub struct Decoder<'formats> {
    attempt_formats: &'formats [Format],
}

impl Default for Decoder<'static> {
    fn default() -> Self {
        Self {
            attempt_formats: &[Format::Track2],
        }
    }
}

/// The result of successfully decoding a magnetic stripe bitstream.
///
/// Contains the decoded data as a string and a reference to the format
/// that was used to successfully decode the data.
#[derive(Debug, Clone, PartialEq)]
pub struct DecoderOutput<'a> {
    /// The decoded character data from the magnetic stripe.
    pub data: String,

    /// Reference to the format that successfully decoded the bitstream.
    /// This allows the caller to know which format from the attempted list worked.
    pub format: &'a Format,
}

/// Errors that can occur during magnetic stripe decoding.
#[derive(Debug, Clone, thiserror::Error)]
pub enum DecoderError {
    /// No formats were provided to attempt decoding.
    #[error("No formats provided for decoding")]
    NoFormatsProvided,

    /// None of the attempted formats could successfully decode the bitstream.
    #[error("Failed to decode with any of the {attempted} provided formats")]
    NoValidFormat {
        /// The number of formats that were attempted.
        attempted: usize,
    },

    /// The bitstream is too short to contain valid data for any format.
    #[error(
        "Bitstream too short: {bit_count} bits provided, but at least {minimum_required} bits required"
    )]
    BitstreamTooShort {
        /// The number of bits in the provided bitstream.
        bit_count: usize,
        /// The minimum number of bits required by the formats.
        minimum_required: usize,
    },

    /// A parity check failed during decoding.
    #[error("Parity check failed at character {position}")]
    ParityError {
        /// The character position where the parity check failed.
        position: usize,
    },

    /// The start sentinel was not found or was invalid.
    #[error("Invalid or missing start sentinel")]
    InvalidStartSentinel,

    /// The end sentinel was not found or was invalid.
    #[error("Invalid or missing end sentinel")]
    InvalidEndSentinel,

    /// The LRC (Longitudinal Redundancy Check) failed.
    #[error("LRC check failed")]
    LrcCheckFailed,

    /// An invalid character was encountered that doesn't match the format's character set.
    #[error("Invalid character at position {position}: {character:?}")]
    InvalidCharacter {
        /// The position where the invalid character was found.
        position: usize,
        /// The invalid character value.
        character: u8,
    },

    /// A custom format specification was invalid or incomplete.
    #[error("Invalid custom format specification: {reason}")]
    InvalidCustomFormat {
        /// Description of what was invalid about the custom format.
        reason: String,
    },
}

impl<'formats> Decoder<'formats> {
    /// Create a new decoder with the specified formats to attempt
    pub fn new(attempt_formats: &'formats [Format]) -> Self {
        Self { attempt_formats }
    }
    
    /// Decode a bitstream using the configured formats
    /// 
    /// This will try each format in order until one succeeds, returning
    /// the decoded data and a reference to the successful format.
    /// If no format succeeds, returns an error indicating the failure.
    pub fn decode(&self, stream: BitStream) -> Result<DecoderOutput<'formats>, DecoderError> {
        decoder::decode_with_formats(self.attempt_formats, stream)
    }
}
