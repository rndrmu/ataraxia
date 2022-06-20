use opus;
use std::{process::Command, fmt::format};
use std::io::{Read, Write};

pub mod vortex_socket;


// some constants, used for audio encoding
pub const SAMPLE_RATE: usize = 44100;
pub const CHANNELS: usize = 2;
pub const FRAME_SIZE: usize = SAMPLE_RATE * 2 / 60;
pub const FRAME_LENGTH: usize = 20; // in milliseconds
pub const SAMPLES_PER_FRAME: usize = SAMPLE_RATE / 1000 * FRAME_LENGTH;
pub const MAX_PACKET_SIZE: usize = FRAME_SIZE * 8;

/// Split the input data into packets of size `MAX_PACKET_SIZE


pub fn encode_to_opus(data: Vec<u8>) -> Result<Vec<u8>, opus::Error> {
    let mut output: Vec<u8> = Vec::new();
    // convert data to a [i16] array so we can use audiopus' encoder
    let input: Vec<i16> = data.chunks_exact(2).into_iter().map(|a| i16::from_ne_bytes([a[0], a[1]])).collect();

    let mut encoder = opus::Encoder::new(SAMPLE_RATE as u32, opus::Channels::Stereo, opus::Application::Voip)?;

    // We'll ignore the output and bubble the error up.
    let res = encoder.encode_vec(&input, MAX_PACKET_SIZE)?;



    Ok(res)

}