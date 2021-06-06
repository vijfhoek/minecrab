use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum BlockType {
    Cobblestone = 1,
    Dirt = 2,
    Stone = 3,
    Grass = 4,
    Bedrock = 5,
    Sand = 6,
    Gravel = 7,
    Water = 8,
}

impl BlockType {
    #[rustfmt::skip]
    pub const fn texture_indices(self) -> (usize, usize, usize, usize, usize, usize) {
        match self {
            BlockType::Cobblestone => ( 0,  0,  0,  0,  0,  0),
            BlockType::Dirt        => ( 1,  1,  1,  1,  1,  1),
            BlockType::Stone       => ( 2,  2,  2,  2,  2,  2),
            BlockType::Grass       => ( 4,  4,  4,  4,  2,  3),
            BlockType::Bedrock     => ( 5,  5,  5,  5,  5,  5),
            BlockType::Sand        => ( 6,  6,  6,  6,  6,  6),
            BlockType::Gravel      => ( 7,  7,  7,  7,  7,  7),
            BlockType::Water       => ( 8,  8,  8,  8,  8,  8), // up to 71
        }
    }

    pub const fn is_transparent(self) -> bool {
        matches!(self, BlockType::Water)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Block {
    pub block_type: BlockType,
}
