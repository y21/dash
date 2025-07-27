use dash_middle::interner::Symbol;
use dash_proc_macro::Trace;

use crate::frame::Frame;

pub const MAX_FRAME_COUNT: usize = 1024;

#[derive(Debug, Clone, Copy, Trace, PartialEq, Eq, PartialOrd, Ord)]
pub struct FrameId(pub u32);

#[derive(Trace)]
pub struct FrameStack {
    frames: Vec<Frame>,
}

impl FrameStack {
    pub fn new() -> Self {
        Self { frames: Vec::new() }
    }

    pub fn current(&self) -> &Frame {
        self.frames.last().expect("no active frame")
    }

    pub fn current_id(&self) -> FrameId {
        FrameId(self.frames.len() as u32 - 1)
    }

    pub fn current_mut(&mut self) -> &mut Frame {
        self.frames.last_mut().expect("no active frame")
    }

    pub fn pop(&mut self) -> Frame {
        self.frames.pop().expect("no active frame")
    }

    /// "Unwinds" to a frame, i.e. removing all frames above the specified frame.
    pub fn unwind_to(&mut self, frame_id: FrameId) {
        self.frames.drain(frame_id.0 as usize + 1..);
    }

    pub fn len(&self) -> u32 {
        self.frames.len() as u32
    }

    pub fn push(&mut self, frame: Frame) -> Result<(), ()> {
        if self.frames.len() < MAX_FRAME_COUNT {
            self.frames.push(frame);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn function_name_iter(&self) -> impl DoubleEndedIterator<Item = Option<Symbol>> {
        self.frames.iter().map(|frame| frame.function.name)
    }
}
