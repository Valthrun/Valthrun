use std::vec::Vec;

pub trait SearchPattern {
    fn length(&self) -> usize;
    fn is_matching(&self, target: &[u8]) -> bool;

    fn find(&self, buffer: &[u8]) -> Option<usize> {
        if self.length() > buffer.len() {
            return None;
        }

        for (index, window) in buffer.windows(self.length()).enumerate() {
            if !self.is_matching(window) {
                continue;
            }

            return Some(index as usize);
        }

        None
    }
}

#[derive(Debug)]
pub enum BytePattern {
    Any,
    Value(u8),
}

impl BytePattern {
    pub fn matches_byte(&self, target: u8) -> bool {
        match self {
            BytePattern::Any => true,
            BytePattern::Value(expected) => target == *expected,
        }
    }

    pub fn parse(pattern: &str) -> Option<BytePattern> {
        if pattern == "?" || pattern == "??" {
            Some(BytePattern::Any)
        } else if let Ok(value) = u8::from_str_radix(pattern, 16) {
            Some(BytePattern::Value(value))
        } else {
            None
        }
    }
}

impl SearchPattern for BytePattern {
    fn length(&self) -> usize {
        1
    }

    fn is_matching(&self, target: &[u8]) -> bool {
        self.matches_byte(target[0])
    }
}

#[derive(Debug)]
pub struct ByteSequencePattern {
    bytes: Vec<BytePattern>,
}

impl ByteSequencePattern {
    pub fn parse(pattern: &str) -> Option<ByteSequencePattern> {
        pattern
            .split(" ")
            .map(BytePattern::parse)
            .collect::<Option<Vec<_>>>()
            .map(|bytes| Self { bytes })
    }
}

impl SearchPattern for ByteSequencePattern {
    fn length(&self) -> usize {
        self.bytes.len()
    }

    fn is_matching(&self, target: &[u8]) -> bool {
        self.bytes
            .iter()
            .zip(target.iter())
            .find(|(pattern, value)| !pattern.matches_byte(**value))
            .is_none()
    }
}
