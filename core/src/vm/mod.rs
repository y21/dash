use self::frame::Frame;

mod frame;
mod value;

pub struct Vm {
    frames: Vec<Frame>,
}

impl Vm {}
