use cgmath::Vector4;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum BlockType {
    Cobblestone,
    Dirt,
    Stone,
    Grass,
    Bedrock,
    Sand,
    Gravel,
    Water,
    OakLog,
    OakPlanks,
    OakLeaves,
}

impl BlockType {
    #[rustfmt::skip]
    pub const fn texture_indices(self) -> (usize, usize, usize, usize, usize, usize) {
        match self {
            BlockType::Cobblestone => ( 0,  0,  0,  0,  0,  0),
            BlockType::Dirt        => ( 1,  1,  1,  1,  1,  1),
            BlockType::Stone       => ( 2,  2,  2,  2,  2,  2),
            BlockType::Grass       => ( 4,  4,  4,  4,  1,  3),
            BlockType::Bedrock     => ( 5,  5,  5,  5,  5,  5),
            BlockType::Sand        => ( 6,  6,  6,  6,  6,  6),
            BlockType::Gravel      => ( 7,  7,  7,  7,  7,  7),
            BlockType::Water       => ( 8,  8,  8,  8,  8,  8), // up to 39
            BlockType::OakLog      => (40, 40, 40, 40, 41, 41),
            BlockType::OakPlanks   => (42, 42, 42, 42, 42, 42),
            BlockType::OakLeaves   => (43, 43, 43, 43, 43, 43),
        }
    }

    pub const fn color(self) -> Vector4<f32> {
        match self {
            Self::Water => Vector4::new(0.1540, 0.2885, 0.5575, 1.0),
            Self::OakLeaves => Vector4::new(0.4784, 0.7294, 0.1255, 1.0),
            _ => Vector4::new(1.0, 1.0, 1.0, 1.0),
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
