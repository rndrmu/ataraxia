use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum VoiceMode {
    /// CombinedWebRtc — sendv and recv are in one 
    CombinedWebRtc,
    /// SplitWebRtc — sendv and recv are separate
    SplitWebRtc,
    /// CombinedRtp — The default mode, UDP is used for sending and receiving.
    CombinedRtp,
}

#[derive(Serialize, Deserialize)]
pub enum RtpHeaderExtensionKind {
    #[serde(rename = "audio")]
    Audio,
    #[serde(rename = "video")]
    Video,
}

#[derive(Serialize, Deserialize)]
pub struct InitializeTransportsPayload {
    pub id: u16,
    #[serde(rename = "type")]
    pub payload_type: String,
    pub data: InitializeTransportsPayloadData,
}

#[derive(Serialize, Deserialize)]
pub struct InitializeTransportsPayloadData {
    pub mode: VoiceMode,
    #[serde(rename = "rtpCapabilities")]
    pub rtp_capabilities: RtpCapabilities,
}

#[derive(Serialize, Deserialize)]
pub struct RtpCapabilities {
    #[serde(rename = "headerExtensions")]
    pub header_extensions: Vec<RtpHeaderExtension>,
    pub codecs: Vec<RtpCodecCapability>,
}

#[derive(Serialize, Deserialize)]
pub struct RtpHeaderExtension {
    pub uri: String,
    pub kind: RtpHeaderExtensionKind,
    #[serde(rename = "preferredId")]
    pub preferred_id: u16,
    #[serde(rename = "preferredEncrypt")]
    pub preferred_encrypt: bool,
    pub direction: String,
}

#[derive(Serialize, Deserialize)]
pub struct RtpCodecCapability {
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub kind: RtpHeaderExtensionKind,
    #[serde(rename = "preferredPayloadType")]
    pub preferred_payload_type: u16,
    pub channels: u8,
    pub parameters: RtpCodecParameter,
    #[serde(rename = "rtcpFeedback")]
    pub rtcp_feedback: Vec<RtcpFeedback>,
}

#[derive(Serialize, Deserialize)]
pub struct RtcpFeedback {
    #[serde(rename = "type")]
    pub rtcp_feedback_type: String,
    pub parameter: String,
}


#[derive(Serialize, Deserialize)]
pub struct RtpCodecParameter {
    pub minptime: u16,
    pub useinbandfec: u8,
}