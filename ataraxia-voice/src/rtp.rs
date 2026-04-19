use byteorder::{BigEndian, WriteBytesExt};

pub struct RtpPacket {
    pub sequence: u16,
    pub timestamp: u32,
    pub ssrc: u32,
    /// RTP payload type (e.g., 120 for Opus in Vortex)
    pub payload_type: u8,
    pub payload: Vec<u8>,
}

impl RtpPacket {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(12 + self.payload.len());
        // V=2, P=0, X=0, CC=0
        buf.push(0x80);
        // M=0, PT
        buf.push(self.payload_type & 0x7F);
        buf.write_u16::<BigEndian>(self.sequence).unwrap();
        buf.write_u32::<BigEndian>(self.timestamp).unwrap();
        buf.write_u32::<BigEndian>(self.ssrc).unwrap();
        buf.extend_from_slice(&self.payload);
        buf
    }
}
