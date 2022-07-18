

/**
 * The following Code is largely based on the code by @melike2D on GitHub:
 * https://github.com/mixtape-bot/voice-testing/
 * The code is licensed under the MIT License.
 * 
 * [melike2D](https://github.com/melike2D)
*/


/* use crate::vortex_socket::RtpHeader; */

#[macro_use]
use crate::yeet;


pub const SUITE_NAME: &str = "aes_cm_hmac_sha1_80";

pub const COUNTER_LENGTH: usize = 16;
pub const SALT_LENGTH: usize = 14;
pub const AUTH_KEY_LENGTH: usize = 20;
pub const AUTH_TAG_LENGTH: usize = 10;
pub const KEY_LENGTH: usize = 16;



// encode to base64 without any external dependencies

/* pub fn create(enc_context: Context, header:RtpHeader, payload: &[u8]) {
    let roc = 0;

    let mut counter_cursor = ByteWriter::with_size(COUNTER_LENGTH, Some(Endianess::Big));

    EncryptionStrategy::encrypt(&mut counter_cursor, header, payload);

}
 */

pub fn encode_base64(data: &[u8]) -> String {
    base64::encode(&data)
}


pub struct Context {
    session: SessionContext
}

pub struct SessionContext {
    pub encryption_key: [u8; KEY_LENGTH],
    pub authentication_key: [u8; AUTH_KEY_LENGTH],
    pub salt: [u8; SALT_LENGTH],
}

pub struct MasterKey {
    pub secret: [u8; KEY_LENGTH],
    pub secret_bytes: Vec<u8>,
    pub salt: [u8; SALT_LENGTH],
}

impl MasterKey {
    pub fn get_base64(&self) -> String {
        let bytes = self.secret_bytes.iter().cloned().chain(self.salt.iter().cloned()).collect::<Vec<u8>>();
        encode_base64(&bytes)
    }
}

/// The byte order the [`ByteWriter`] should write as 
/// 
/// [`ByteWriter`]: struct.ByteWriter.html
pub enum Endianess {
    /// Little endian Order — Least significant byte first
    Little,
    /// Big endian Order — Most significant byte first
    /// 
    /// This is the default order.
    Big,
}

/// A writer that writes to a byte buffer.
/// 
/// Example: A writer with byte order `Big` and a capacity of `10` will write to a buffer of length `10`.
/// ```rs
/// // To instantiate a writer with byte order `Big` and a capacity of `10`:
/// // `Some(Endianess::Big)` can also be `None` as the Byteorder is Big Endian by default.
/// let mut writer = ByteWriter::with_size(10, Some(Endianess::Big));
/// // To write a byte of size `[u32]` to the buffer:
/// writer.write_u32(420 >> 69);
/// ```
/// 
/// [`ByteWriter::with_size`]: struct.ByteWriter.html#method.with_size
/// [`ByteWriter::write_u32`]: struct.ByteWriter.html#method.write_u32
/// [`ByteWriter::write_u16`]: struct.ByteWriter.html#method.write_u16
/// [`u32`]: https://doc.rust-lang.org/std/primitive.u32.html
pub struct ByteWriter {
    /// The buffer to write to.
    pub data: Vec<u8>,
    /// The byte order to write as.
    /// 
    /// Default is `Big`.
    pub endianess: Endianess,
}

impl ByteWriter {
    /// Resets the writer to its initial state.
    fn reset(&mut self) {
        self.data.clear();
    }

    /// Creates a new writer with a given capacity [`size`].
    /// 
    /// [`size`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.with_capacity
    fn with_size(size: usize, endianess: Option<Endianess>) -> Self {
        Self {
            data: vec![0; size],
            endianess: endianess.unwrap_or(Endianess::Big), // default to big endian
        }
    }

    /// Resizes the buffer to a new capacity.
    fn resize(&mut self, new_size: usize) -> &mut Self {
        self.data.resize(new_size, 0);
        self
    }


    /// Writes a [`u16`] to the buffer.
    /// 
    /// [`u16`]: https://doc.rust-lang.org/std/primitive.u16.html
    fn write_u16(&mut self, value: u16, endianess: Option<Endianess>) {
        match endianess {
            Some(Endianess::Little) => {
                let bytes = value.to_le_bytes();
                self.data.extend_from_slice(&bytes);
            }
            Some(Endianess::Big) | None => {
                let bytes = value.to_be_bytes().into_iter().rev().take(2).collect::<Vec<u8>>(); // :^) only take non-zero bytes
                self.data.extend_from_slice(&bytes);
            }
        }
    }

    /// Writes a [`u32`] to the buffer.
    /// 
    /// [`u32`]: https://doc.rust-lang.org/std/primitive.u32.html
    fn write_u32(&mut self, value: u32, endianess: Option<Endianess>) {
        match endianess {
            // This is mostly for discord, however it's nice to be feature-complete. :) 
            Some(Endianess::Little) => {
                let bytes = value
                    .to_le_bytes(); // :^) only take non-zero bytes
                self.data.extend_from_slice(&bytes);
            }
            // Big Endian is default
            Some(Endianess::Big) | None => {
                let bytes = value.to_be_bytes(); // :^) only take non-zero bytes
                self.data.extend_from_slice(&bytes);
            }
        }
    }

    fn is_not_full(&self) -> Result<(), String> {
        if self.data.len() == self.data.capacity() {
            yeet!("Buffer is full".to_string());
        } else {
            Ok(())
        }
    }
}

pub struct EncryptionStrategy;

impl EncryptionStrategy {

   /*  pub fn encrypt(
        cursor: &mut ByteWriter,
        header: RtpHeader,
        payload: &[u8],
    ) {

    } */

    pub fn next_sequence(prev: u16) -> u16 {
        prev + 1
    }
}


fn generate_counter(
    cursor: &mut ByteWriter,
    seq: u16,
    roc: u32,
    ssrc: u32,
    salt: &[u8],
) {
    cursor.write_u32(0, None);
    cursor.write_u32(ssrc, None);
    cursor.write_u32(roc, None);
    cursor.write_u32((seq << 16) as u32, None);

    for i in 0..salt.len() {
        cursor.data[i] ^= salt[i];
    }
    
}

fn generate_hmac_auth_tag(
    key: &[u8],
    input: &[u8],
    roc: u32, 
) {

}