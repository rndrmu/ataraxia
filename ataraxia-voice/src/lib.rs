use std::io::{Read, Write};
use std::{fmt::format, process::Command};

pub mod vortex_socket;
pub mod crypto;

#[macro_use]
pub mod macros;


// some constants, used for audio encoding
pub const SAMPLE_RATE: usize = 44100;
pub const CHANNELS: usize = 2;
pub const FRAME_SIZE: usize = SAMPLE_RATE * 2 / 60;
pub const FRAME_LENGTH: usize = 20; // in milliseconds
pub const SAMPLES_PER_FRAME: usize = SAMPLE_RATE / 1000 * FRAME_LENGTH;
pub const MAX_PACKET_SIZE: usize = FRAME_SIZE * 8;

