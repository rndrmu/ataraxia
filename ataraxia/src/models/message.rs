use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub _id: String,
    pub author: String,
    pub channel: String,
    pub content: String,
    pub nonce: String,
    pub mentions: Option<Vec<String>>,
    pub attachments: Option<Vec<MessageAttachments>>,
    pub edited: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct  MessageAttachments {
    pub _id: String,
    pub tag: String,
    pub filename: String,
    pub metadata: MessageMetadata,
    pub content_type: String,
    pub size: usize
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageMetadata {
    #[serde(rename = "type")]
    pub _type: String,
    pub width: usize,
    pub height: usize,
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.content.is_empty() {
            write!(f, "Channel: {}, Author: {}", self.channel, self.author)
        } else {
            write!(f, "Channel: {}, Author: {}, Content: {}", self.channel, self.author, self.content)
        }
    }
}
