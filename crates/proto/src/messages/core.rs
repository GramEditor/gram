use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub struct PeerId {
    pub owner_id: u32,
    pub id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct User {
    pub id: u64,
    pub email: String,
    pub avatar_url: String,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Nonce {
    pub upper_half: u64,
    pub lower_half: u64,
}
