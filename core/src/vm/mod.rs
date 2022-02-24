use std::convert::TryInto;

use crate::{
    gc::{handle::Handle, Gc},
    js_std,
};

use self::{
    dispatch::HandleResult,
    frame::Frame,
    value::{
        function::{Function, FunctionKind},
        object::{AnonymousObject, Object},
        Value,
    },
};

pub mod dispatch;
mod frame;
pub mod value;

pub const MAX_STACK_SIZE: usize = 8196;

pub struct Vm {
    frames: Vec<Frame>,
    stack: Vec<Value>,
    gc: Gc<dyn Object>,
    global: Handle<dyn Object>,
}

impl Vm {
    pub fn new() -> Self {
        let mut gc = Gc::new();
        let global = gc.register(AnonymousObject::new());

        let mut vm = Self {
            frames: Vec::new(),
            stack: Vec::with_capacity(512),
            gc,
            global,
        };
        vm.prepare();
        vm
    }

    /// Prepare the VM for execution.
    #[rustfmt::skip]
    fn prepare(&mut self) {
        let global = self
            .global
            .as_any()
            .downcast_ref::<AnonymousObject>()
            .expect("global is not an object");

        let log = Function::new("log".into(), FunctionKind::Native(js_std::global::log));
        global.set_property("log", self.gc.register(log).into()).unwrap();
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
        let frame = self.frames.last_mut().expect("No frame");
        let value = Value::from_constant(frame.constants[idx].clone());
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
#[cfg(test)]
mod testing {
    use crate::{compiler::FunctionCompiler, parser::parser::Parser};

    use super::*;

    #[test]
    fn vm() {
        let mut vm = Vm::new();

        let ast = Parser::from_str("log(12+34)")
            .unwrap()
            .parse_all(true)
            .unwrap();

        let compiled = FunctionCompiler::compile_ast(ast).unwrap();

        let frame = Frame {
            buffer: compiled.instructions.into_boxed_slice(),
            ip: 0,
            constants: compiled.cp.into_vec().into_boxed_slice(),
        };

        println!("{:?}", vm.execute_frame(frame));
    }
}
