mod frame;
mod message;

pub use frame::{FrameError, MAX_FRAME_BYTES, decode_frame, read_frame, write_frame};
pub use message::{
    AudioChunk, Command, Envelope, Event, FileTranscription, ModelDescriptor, StreamDescriptor,
};

pub const PROTOCOL_VERSION: u16 = 1;
