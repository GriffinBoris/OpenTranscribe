mod frame;
mod message;

pub use frame::{FrameError, MAX_FRAME_BYTES, decode_frame, read_frame, write_frame};
pub use message::{Command, Envelope, Event, FileTranscription, ModelDescriptor};

pub const PROTOCOL_VERSION: u16 = 3;
