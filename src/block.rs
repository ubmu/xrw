use crate::Mark;
use std::fmt;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Type {
    Standard { offset: u64, payload_offset: u64, payload_size: u64, padding: u64 },
    Custom(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub mark: Mark,
    pub _type: Type,
    //pub subtype: Option<Mark>,
    // pub children: Vec<Block>,
}

impl Block {
    /// Creates a block with a custom payload to be inserted into a file.
    pub fn custom(mark: Mark, payload: Vec<u8>) -> Self {
        Self { mark: mark, _type: Type::Custom(payload) }
    }

    pub fn is_standard(&self) -> bool {
        matches!(self._type, Type::Standard { .. })
    }

    pub fn is_custom(&self) -> bool {
        matches!(self._type, Type::Custom(_))
    }

    pub fn offset(&self) -> Option<u64> {
        match &self._type {
            Type::Standard { offset, .. } => Some(*offset),
            Type::Custom { .. } => None,
        }
    }

    pub fn payload_offset(&self) -> Option<u64> {
        match &self._type {
            Type::Standard { payload_offset, .. } => Some(*payload_offset),
            Type::Custom { .. } => None,
        }
    }

    pub fn payload_size(&self) -> u64 {
        match &self._type {
            Type::Standard { payload_size, .. } => *payload_size,
            Type::Custom(payload) => payload.len() as u64,
        }
    }

    //pub fn is_container(&self) -> bool {
    //    !self.children.is_empty()
    //}

    pub fn find_child(&self, mark: Mark) -> Option<&Block> {
        todo!()
    }

    pub fn find_all_children(&self, mark: Mark) -> Vec<&Block> {
        todo!()
    }

    // REMOVE THIS FUNCTION AFTER BLOCK.RS IS PUSHED TO STATE REASON.
    // WILL NEVER BE CALLED IN WRITE FUNCTIONS AS THEY WILL CALCULATE PADDING.
    //
    // TODO: IF THIS IS REMOVED, SHOULD I KEEP STORING PADDING IN BLOCK?
    pub(crate) fn padding(&self) -> u64 {
        match &self._type {
            Type::Standard { padding, .. } => *padding,
            Type::Custom(..) => 0,
        }
    }

    pub(crate) fn custom_payload(&self) -> Option<&[u8]> {
        match &self._type {
            Type::Custom(payload) => Some(payload),
            _ => unreachable!(
                "This function will never be called on Type::Standard blocks."
            ),
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self._type {
            Type::Standard { offset, payload_offset, payload_size, padding } => {
                write!(
                    f,
                    "[{}] [offset: {}, payload_offset: {}, payload_size: {}, padding: {}]",
                    self.mark, offset, payload_offset, payload_size, padding
                )
            }
            Type::Custom(payload) => {
                write!(f, "{} [type: custom, size: {}]", self.mark, payload.len())
            }
        }
    }
}
