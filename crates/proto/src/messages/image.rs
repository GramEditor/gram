use serde::{Deserialize, Serialize};
use std::default::Default;

use crate::messages::core::PeerId;
use crate::messages::worktree::File;

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OpenImageByPath {
    pub project_id: u64,
    pub worktree_id: u64,
    pub path: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OpenImageResponse {
    pub image_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum CreateImageForPeerVariant {
    State(ImageState),
    Chunk(ImageChunk),
}
impl Default for CreateImageForPeerVariant {
    fn default() -> Self {
        Self::State(Default::default())
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CreateImageForPeer {
    pub project_id: u64,
    pub peer_id: PeerId,
    pub variant: CreateImageForPeerVariant,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ImageState {
    pub id: u64,
    pub file: Option<File>,
    pub content_size: u64,
    pub format: String, // e.g., "png", "jpeg", "webp", etc.
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ImageChunk {
    pub image_id: u64,
    pub data: Vec<u8>,
}
