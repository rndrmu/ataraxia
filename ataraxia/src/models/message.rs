use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::Value;
#[derive(Serialize, Deserialize, Debug)]
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

// Create a builder for a message
#[derive(Serialize, Debug)]
pub struct CreateMessage (
    #[serde(borrow)]
    pub HashMap<&'static str, Value>,
);

#[derive(Serialize, Deserialize, Debug)]
struct MasqueradeMessage {
    pub name: String,
    pub avatar: String,
}

impl CreateMessage {
     /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    #[inline]
    pub fn content<D: ToString>(&mut self, content: D) -> &mut Self {
        self._content(content.to_string())
    }

    fn _content(&mut self, content: String) -> &mut Self {
        self.0.insert("content", Value::from(content));
        self
    }

    pub fn masquerade(&mut self, name: &str, avatar: &str) -> &mut Self {
        self.0.insert("masquerade", serde_json::to_value(MasqueradeMessage {
            name: name.to_string(),
            avatar: avatar.to_string(),
        }).unwrap());
        self
    }
}



impl Default for CreateMessage {
    fn default() -> CreateMessage {
        let mut map = HashMap::new();
        map.insert("content", Value::from("hello ataraxia!"));

        CreateMessage(map)
    }
}

use std::error::Error as StdError;
use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, serde_json::Error>;
pub type JsonMap = serde_json::Map<String, Value>;

#[cfg(not(feature = "simd-json"))]
pub const NULL: Value = Value::Null;
#[cfg(feature = "simd-json")]
pub const NULL: Value = Value::Static(simd_json::StaticNode::Null);

/// Converts a HashMap into a final [`JsonMap`] representation.
pub fn hashmap_to_json_map<H, T>(map: HashMap<T, Value, H>) -> JsonMap
where
    H: std::hash::BuildHasher,
    T: Eq + std::hash::Hash + ToString,
{
    map.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
}

pub(crate) fn to_value<T>(value: T) -> Result<Value>
where
    T: Serialize,
{
    Ok(serde_json::to_value(value)?)
}