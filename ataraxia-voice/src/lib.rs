use audiopus;
use byteorder::{BigEndian, ReadBytesExt}; // 1.2.7

pub mod vortex_socket;


// some constants, used for audio encoding
pub const SAMPLE_RATE: usize = 44100;
pub const CHANNELS: usize = 2;
pub const FRAME_SIZE: usize = SAMPLE_RATE * 2 / 60;
pub const FRAME_LENGTH: usize = 20; // in milliseconds
pub const SAMPLES_PER_FRAME: usize = SAMPLE_RATE / 1000 * FRAME_LENGTH;


pub fn encode_to_opus(data: &[u8]) -> Result<Vec<u8>, audiopus::Error> {
    let mut output: Vec<u8> = Vec::new();
    // convert data to a [i16] array so we can use audiopus' encoder
    let mut input: Vec<i16> = Vec::new();
    for i in 0..data.len() {
        input.push(data[i] as i16);
    }

    let encoder = audiopus::coder::Encoder::new(audiopus::SampleRate::Hz48000, audiopus::Channels::Stereo, audiopus::Application::Voip)?;

    // We'll ignore the output and bubble the error up.
    let _ = audiopus::coder::Encoder::encode(&encoder, &input, &mut output)?;

    Ok(output)
}