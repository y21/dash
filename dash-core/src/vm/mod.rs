use std::{convert::TryInto, fmt};

use crate::gc::{handle::Handle, Gc};

use self::{
    dispatch::HandleResult,
    external::Externals,
    frame::Frame,
    local::LocalScope,
    statics::Statics,
    value::{
        object::{NamedObject, Object},
        Value,
    },
};

pub mod dispatch;
pub mod external;
pub mod frame;
pub mod local;
pub mod statics;
pub mod value;

pub const MAX_STACK_SIZE: usize = 8196;

pub struct Vm {
    frames: Vec<Frame>,
    stack: Vec<Value>,
    gc: Gc<dyn Object>,
    global: Handle<dyn Object>,
    externals: Externals,
    statics: Statics,
}

impl Vm {
    pub fn new() -> Self {
        let mut gc = Gc::new();
        let statics = Statics::new(&mut gc);
        let global = gc.register(NamedObject::new());

        let mut vm = Self {
            frames: Vec::new(),
            stack: Vec::with_capacity(512),
            gc,
            global,
            externals: Externals::new(),
            statics,
        };
        vm.prepare();
        vm
    }

    /// Prepare the VM for execution.
    #[rustfmt::skip]
    fn prepare(&mut self) {
        let mut scope = LocalScope::new(self);

        let global = scope.global.clone();

        let console = {
            let console = scope.statics.console.clone();
            let log = scope.statics.log.clone();
            console.set_property(&mut scope, "log", log.into()).unwrap();
            console
        };

        let math = {
            let math = scope.statics.math.clone();
            let floor = scope.statics.floor.clone();
            math.set_property(&mut scope, "floor", floor.into()).unwrap();
            math
        };

        global.set_property(&mut scope, "console", console.into()).unwrap();
        global.set_property(&mut scope, "Math", math.into()).unwrap();
    }

    /// Fetches the current instruction/value in the currently executing frame
    /// and increments the instruction pointer
    pub(crate) fn fetch_and_inc_ip(&mut self) -> u8 {
        let frame = self.frames.last_mut().expect("No frame");
        let ip = frame.ip;
        frame.ip += 1;
        frame.buffer[ip]
    }

    /// Fetches a wide value (16-bit) in the currently executing frame
    /// and increments the instruction pointer
    pub(crate) fn fetchw_and_inc_ip(&mut self) -> u16 {
        let frame = self.frames.last_mut().expect("No frame");
        let value: [u8; 2] = frame.buffer[frame.ip..frame.ip + 2]
            .try_into()
            .expect("Failed to get wide instruction");

        frame.ip += 2;
        u16::from_ne_bytes(value)
    }

    /// Pushes a constant at the given index in the current frame on the top of the stack
    pub(crate) fn push_constant(&mut self, idx: usize) -> Result<(), Value> {
        let frame = self.frames.last().expect("No frame");
        let value = Value::from_constant(frame.constants[idx].clone(), self);
        self.try_push_stack(value)?;
        Ok(())
    }

    pub(crate) fn try_push_stack(&mut self, value: Value) -> Result<(), Value> {
        if self.stack.len() > MAX_STACK_SIZE {
            panic!("Stack overflow"); // todo: return result
        }
        self.stack.push(value);
        Ok(())
    }

    /// Executes a frame in this VM
    pub fn execute_frame(&mut self, frame: Frame) -> Result<Value, Value> {
        self.stack
            .resize(self.stack.len() + frame.local_count, Value::Undefined);

        self.frames.push(frame);

        loop {
            let instruction = self.fetch_and_inc_ip();

            match dispatch::handle(self, instruction) {
                Ok(HandleResult::Return(value)) => return Ok(value),
                Ok(HandleResult::Continue) => continue,
                Err(e) => return Err(e),
            }
        }
    }
}

impl fmt::Debug for Vm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Vm")
    }
}
