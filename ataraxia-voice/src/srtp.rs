use aes::Aes128;
use cipher::{KeyIvInit, StreamCipher};
use ctr::Ctr128BE;
use hmac::{Hmac, Mac};
use sha1::Sha1;

type HmacSha1 = Hmac<Sha1>;

pub struct SrtpContext {
    cipher_key: [u8; 16],
    auth_key: [u8; 20],
    session_salt: [u8; 14],
    rollover_counter: u32,
    ssrc: u32,
    last_seq: u16,
}

impl SrtpContext {
    pub fn new(master_key: [u8; 16], master_salt: [u8; 14], ssrc: u32) -> Self {
        let cipher_key: [u8; 16] = prf_aes_cm(&master_key, &make_x(&master_salt, 0x00), 16)
            .try_into()
            .unwrap();
        let auth_key: [u8; 20] = prf_aes_cm(&master_key, &make_x(&master_salt, 0x01), 20)
            .try_into()
            .unwrap();
        let session_salt: [u8; 14] = prf_aes_cm(&master_key, &make_x(&master_salt, 0x02), 14)
            .try_into()
            .unwrap();
        SrtpContext {
            cipher_key,
            auth_key,
            session_salt,
            rollover_counter: 0,
            ssrc,
            last_seq: 0,
        }
    }

    /// Encrypt and authenticate an RTP packet (AES_CM_128_HMAC_SHA1_80).
    /// The packet must have a standard 12-byte fixed header.
    pub fn protect(&mut self, rtp: &[u8]) -> Vec<u8> {
        assert!(rtp.len() >= 12, "RTP packet too short");

        let seq = u16::from_be_bytes([rtp[2], rtp[3]]);
        // Detect sequence wrap-around and increment ROC
        if self.last_seq > 0xC000 && seq < 0x4000 {
            self.rollover_counter = self.rollover_counter.wrapping_add(1);
        }
        self.last_seq = seq;

        let packet_index = (self.rollover_counter as u64) << 16 | seq as u64;
        let iv = compute_iv(&self.session_salt, self.ssrc, packet_index);

        let mut protected = rtp.to_vec();

        // AES-CM: encrypt payload only (skip 12-byte header)
        let mut cipher = Ctr128BE::<Aes128>::new(&self.cipher_key.into(), &iv.into());
        cipher.apply_keystream(&mut protected[12..]);

        // HMAC-SHA1-80: authenticate the encrypted packet + ROC
        let mut mac = HmacSha1::new_from_slice(&self.auth_key).expect("HMAC init");
        mac.update(&protected);
        mac.update(&self.rollover_counter.to_be_bytes());
        let tag = mac.finalize().into_bytes();

        protected.extend_from_slice(&tag[..10]);
        protected
    }
}

/// SRTP IV = (salt * 2^16) XOR (ssrc * 2^64) XOR (index * 2^16)
/// In big-endian bytes [0..16]:
///   bytes [0:4]  = salt[0:4]
///   bytes [4:8]  = salt[4:8]  XOR ssrc
///   bytes [8:14] = salt[8:14] XOR index (48-bit)
///   bytes [14:16] = 0
fn compute_iv(salt: &[u8; 14], ssrc: u32, packet_index: u64) -> [u8; 16] {
    let mut iv = [0u8; 16];
    iv[0..14].copy_from_slice(salt);

    let ssrc_b = ssrc.to_be_bytes();
    for i in 0..4 {
        iv[4 + i] ^= ssrc_b[i];
    }

    // packet_index is at most 48 bits; treat as u64 and take upper 6 of 8 BE bytes
    let idx_b = packet_index.to_be_bytes();
    for i in 0..6 {
        iv[8 + i] ^= idx_b[2 + i];
    }

    iv
}

/// x = master_salt XOR (label * 2^48)
/// In a 14-byte big-endian value, 2^48 lands on byte index 7 (from MSB).
fn make_x(salt: &[u8; 14], label: u8) -> [u8; 14] {
    let mut x = *salt;
    x[7] ^= label;
    x
}

/// SRTP key derivation using AES-CM (RFC 3711 §4.3.1).
/// PRF: pad x to 16 bytes with 2 leading zeros, then use AES-CM.
fn prf_aes_cm(master_key: &[u8; 16], x: &[u8; 14], length: usize) -> Vec<u8> {
    let mut iv = [0u8; 16];
    iv[2..16].copy_from_slice(x);

    let mut output = vec![0u8; length];
    let mut cipher = Ctr128BE::<Aes128>::new(master_key.into(), &iv.into());
    cipher.apply_keystream(&mut output);
    output
}
