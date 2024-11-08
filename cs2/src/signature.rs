use crate::{
    ByteSequencePattern,
    SearchPattern,
};

pub enum SignatureType {
    /// The value is an address relative to the current instruction.
    /// When resolved the absolute address the instruction pointed towards will be returned.
    RelativeAddress { inst_length: u64 },

    /// The value is an offset within a struct.
    /// (Offsets are assumed to be u32)
    Offset,
}

/// A signature which leads to an offset or address
/// based on a sequence of instructions.
pub struct Signature {
    pub debug_name: String,
    pub pattern: Box<dyn SearchPattern>,
    pub offset: u64,
    pub value_type: SignatureType,
}

impl Signature {
    /// Create a new relative address signature from a byte sequence pattern.
    /// Note: If the pattern is invalid this will panic!
    pub fn relative_address(
        debug_name: impl Into<String>,
        pattern: &str,
        offset: u64,
        inst_length: u64,
    ) -> Self {
        let pattern = Box::new(ByteSequencePattern::parse(pattern).expect("to be a valid pattern"));

        Self {
            debug_name: debug_name.into(),
            pattern,
            offset,
            value_type: SignatureType::RelativeAddress { inst_length },
        }
    }

    pub fn offset(debug_name: impl Into<String>, pattern: &str, offset: u64) -> Self {
        let pattern = Box::new(ByteSequencePattern::parse(pattern).expect("to be a valid pattern"));

        Self {
            debug_name: debug_name.into(),
            pattern,
            offset,
            value_type: SignatureType::Offset,
        }
    }
}
