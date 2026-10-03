use serde::{Serialize, Deserialize};

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetNotifications {
    pub before_id: Option<u64>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AddNotification {
    pub notification: Notification,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetNotificationsResponse {
    pub notifications: Vec<Notification>,
    pub done: bool,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DeleteNotification {
    pub notification_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpdateNotification {
    pub notification: Notification,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MarkNotificationRead {
    pub notification_id: u64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Notification {
    pub id: u64,
    pub timestamp: u64,
    pub kind: String,
    pub entity_id: Option<u64>,
    pub content: String,
    pub is_read: bool,
    pub response: Option<bool>,
}
