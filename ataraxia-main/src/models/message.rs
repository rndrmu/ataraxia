use reqwest::Result as HTTPResult;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap};
use serde_json::Value;
use crate::http::{Http, API_BASE_URL};

use super::id::{
    UserId,
    ChannelId,
    MessageId,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    #[serde(rename = "_id")]
    pub id: MessageId,
    pub author: UserId,
    #[serde(rename = "channel")]
    pub channel_id: ChannelId,
    pub content: String,
    pub nonce: String,
    pub mentions: Option<Vec<String>>,
    pub attachments: Option<Vec<MessageAttachments>>,
    pub edited: Option<String>,
    pub embed: Option<Vec<Embed>>
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
            write!(f, "Channel: {:?}, Author: {:?}", self.channel_id, self.author)
        } else {
            write!(f, "Channel: {:?}, Author: {:?}, Content: {}", self.channel_id, self.author, self.content)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Embed {
    pub icon_url: Option<String>,
    pub url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub media: Option<String>,
    pub colour: Option<String>,
}
// Create a builder for a message
#[derive(Serialize, Debug)]
pub struct CreateMessage (
    #[serde(borrow)]
    pub HashMap<&'static str, Value>,
);

#[derive(Serialize, Debug)]
pub struct CreateEmbed (
    #[serde(borrow)]
    pub HashMap<&'static str, Value>,
);

#[derive(Serialize, Deserialize, Debug)]
struct MasqueradeMessage {
    pub name: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateMasqueradeMessage {
    name: Option<String>,
    avatar: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PartialMessage {
    // all fields are optional
    pub content: Option<String>,
    pub nonce: Option<String>,
    pub mentions: Option<Vec<String>>,
    pub attachments: Option<Vec<MessageAttachments>>,
    pub edited: Option<String>,
    pub embed: Option<Vec<Embed>>
    
}


impl CreateMessage {
     /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    #[inline]
    pub fn content<D: ToString>(&mut self, content: D) -> &mut Self {
        self.0.insert("content", Value::from(content.to_string()));
        self
    }


    pub fn masquerade(&mut self, name: &str) -> &mut Self {
        self.0.insert("masquerade", serde_json::to_value(MasqueradeMessage {
            name: Some(name.to_string()),
            avatar: None,
        }).unwrap());
        self
    }

    pub fn set_masquerade<F>(&mut self, f: F) -> &mut Self
    where F: FnOnce(&mut CreateMasqueradeMessage) -> &mut CreateMasqueradeMessage {
        let mut mm = CreateMasqueradeMessage::default();
        let masq = f(&mut mm);
        self.0.insert("masquerade", serde_json::to_value(masq).unwrap());
        self
    }

    /// Create an embed in the message. And push it to the embeds array.
    pub fn create_embed<T>(&mut self, f: T) -> &mut Self
    where T: FnOnce(&mut CreateEmbed) -> &mut CreateEmbed {
        let mut embed = CreateEmbed(HashMap::new());
        let embed = f(&mut embed);
        self.0.entry("embeds").or_insert(Value::Array(vec![])).as_array_mut().unwrap().push(to_value(embed).unwrap());
        
        self
    }
}

impl CreateMasqueradeMessage {
    pub fn name<D: ToString>(&mut self, name: D) -> &mut Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn avatar<D: ToString>(&mut self, avatar: D) -> &mut Self {
        self.avatar = Some(avatar.to_string());
        self
    }

}


impl CreateEmbed {
    pub fn title<D: ToString>(&mut self, title: D) -> &mut Self {
        self.0.insert("title", Value::from(title.to_string()));
        self
    }

    pub fn description<D: ToString>(&mut self, description: D) -> &mut Self {
        self.0.insert("description", Value::from(description.to_string()));
        self
    }

    pub fn url<D: ToString>(&mut self, url: D) -> &mut Self {
        self.0.insert("url", Value::from(url.to_string()));
        self
    }

    /// Set the colour of the embed.
    /// 
    /// **Note**: The colour must be a hexadecimal string.
    pub fn colour<D: ToString>(&mut self, colour: D) -> &mut Self {
        self.0.insert("colour", Value::from(colour.to_string()));
        self
    }

    pub fn icon_url<D: ToString>(&mut self, icon_url: D) -> &mut Self {
        self.0.insert("icon_url", Value::from(icon_url.to_string()));
        self
    }

}



impl Default for CreateMessage {
    fn default() -> CreateMessage {
        let mut map = HashMap::new();
        map.insert("title", Value::from("hello ataraxia!"));

        CreateMessage(map)
    }
}

impl Default for CreateEmbed {
    fn default() -> CreateEmbed {
        let mut map = HashMap::new();
        map.insert("content", Value::from("hello ataraxia!"));

        CreateEmbed(map)
    }
}

impl Default for CreateMasqueradeMessage {
    fn default() -> CreateMasqueradeMessage {
        CreateMasqueradeMessage {
            name: None,
            avatar: None,
        }
    }
}






pub type Result<T> = std::result::Result<T, serde_json::Error>;
pub type JsonMap = serde_json::Map<String, Value>;

// null value 
pub const NULL: Value = Value::Null;

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

impl Message {

    /// Edits a given Message, preserving all non-specified fields.
    pub async fn edit<F>(&self, http: &Http, f: F) -> HTTPResult<Message>
    where
        F: FnOnce(&mut CreateMessage) -> &mut CreateMessage,
    {
        let mut message = CreateMessage::default();
        f(&mut message);

        let json = to_value(message).unwrap(); // this should never fail :^) 

        let url = format!("{}/channels/{}/messages/{}", API_BASE_URL, self.channel_id.0, self.id.0);

        let res = http.client.patch(&url)
            .header("x-bot-token", http.token.as_ref().unwrap())
            .json(&json)
            .send()
            .await?
            .json::<Message>()
            .await?;

        Ok(res)

    }
}