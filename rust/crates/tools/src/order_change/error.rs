use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum OrderChangeError {
    LengthMismatch { current: usize, target: usize },
    ContentMismatch,
    HashCollision,
    InvalidData(String),
    DetectionFailed(String),
    ReorderFailed(String),
    NodeNotFound(String),
    InvalidNodeState(String),
}

impl fmt::Display for OrderChangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LengthMismatch { current, target } => {
                write!(
                    f,
                    "Length mismatch: current={} target={}",
                    current, target
                )
            }
            Self::ContentMismatch => {
                write!(f, "Content mismatch between current and target data")
            }
            Self::HashCollision => {
                write!(f, "Hash collision detected")
            }
            Self::InvalidData(msg) => {
                write!(f, "Invalid data: {}", msg)
            }
            Self::DetectionFailed(msg) => {
                write!(f, "Detection failed: {}", msg)
            }
            Self::ReorderFailed(msg) => {
                write!(f, "Reorder failed: {}", msg)
            }
            Self::NodeNotFound(msg) => {
                write!(f, "Node not found: {}", msg)
            }
            Self::InvalidNodeState(msg) => {
                write!(f, "Invalid node state: {}", msg)
            }
        }
    }
}

impl std::error::Error for OrderChangeError {}

impl OrderChangeError {
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::LengthMismatch { .. }
                | Self::ContentMismatch
                | Self::DetectionFailed(_)
                | Self::ReorderFailed(_)
        )
    }

    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            Self::InvalidData(_) | Self::InvalidNodeState(_)
        )
    }
}

pub type Result<T> = std::result::Result<T, OrderChangeError>;
