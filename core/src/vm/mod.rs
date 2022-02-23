use std::convert::TryInto;

use crate::{
    compiler::instruction as opcode,
    gc::{Gc, Handle},
    js_std,
};

use self::{
    frame::Frame,
    value::{
        function::{Function, FunctionKind},
        object::{AnonymousObject, Object},
        Value,
    },
};

mod frame;
pub mod value;

pub const MAX_STACK_SIZE: usize = 8196;

pub struct Vm {
    frames: Vec<Frame>,
    stack: Vec<Value>,
    gc: Gc<Box<dyn Object>>,
    global: Handle<Box<dyn Object>>,
}

impl Vm {
    pub fn new() -> Self {
        let mut gc = Gc::new();
        let mut global = Box::new(AnonymousObject::new()) as Box<dyn Object>;
        {
            let log = Function::new("log".into(), FunctionKind::Native(js_std::global::log));
            let log = gc.register(Box::new(log) as Box<dyn Object>);
            global.set_property("log", Value::Object(log)).unwrap();
        }
        let global = gc.register(global);

        Self {
            frames: Vec::new(),
            stack: Vec::with_capacity(512),
            gc,
            global,
        }
    }

    /// Fetches the current instruction/value in the currently executing frame
    /// and increments the instruction pointer
    fn fetch_and_inc_ip(&mut self) -> u8 {
        let frame = self.frames.last_mut().expect("No frame");
        let ip = frame.ip;
        frame.ip += 1;
        frame.buffer[ip]
    }

    /// Fetches a wide value (16-bit) in the currently executing frame
    /// and increments the instruction pointer
    fn fetchw_and_inc_ip(&mut self) -> u16 {
        let frame = self.frames.last_mut().expect("No frame");
        let value: [u8; 2] = frame.buffer[frame.ip..frame.ip + 2]
            .try_into()
            .expect("Failed to get wide instruction");

        frame.ip += 2;
        u16::from_ne_bytes(value)
    }

    /// Pushes a constant at the given index in the current frame on the top of the stack
    fn push_constant(&mut self, idx: usize) -> Result<(), Value> {
        let frame = self.frames.last_mut().expect("No frame");
        let value = Value::from_constant(frame.constants[idx].clone());
        self.try_push_stack(value)?;
        Ok(())
    }

    fn try_push_stack(&mut self, value: Value) -> Result<(), Value> {
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

            match instruction {
                opcode::CONSTANT => {
                    let id = self.fetch_and_inc_ip();
                    self.push_constant(id as usize)?;
                }
                opcode::CONSTANTW => {
                    let id = self.fetchw_and_inc_ip();
                    self.push_constant(id as usize)?;
                }
                opcode::ADD => {
                    let right = self.stack.pop().expect("No right operand");
                    let left = self.stack.pop().expect("No left operand");
                    self.try_push_stack(left.add(&right))?;
                }
                opcode::POP => {
                    self.stack.pop();
                }
                opcode::RET => {
                    let value = self.stack.pop().expect("No return value");
                    let this = self.frames.pop().expect("No frame");

                    if self.frames.is_empty() {
                        // returning from the last frame means we are done
                        return Ok(value);
                    }
                }
                opcode::LDGLOBAL => {
                    let id = self.fetch_and_inc_ip();
                    let constant = self
                        .frames
                        .last()
                        .expect("No frame")
                        .constants
                        .get(id as usize)
                        .expect("Invalid constant reference in bytecode");

                    let name = constant
                        .as_identifier()
                        .expect("Referenced constant is not an identifier");

                    let value = self.global.get().borrow().get_property(name)?;
                    self.stack.push(value);
                }
                opcode::LDGLOBALW => {}
                opcode::CALL => {
                    let argc = self.fetch_and_inc_ip();
                    let is_constructor = self.fetch_and_inc_ip();

                    let mut args = Vec::with_capacity(argc.into());
                    for _ in 0..argc {
                        args.push(self.stack.pop().expect("Missing argument"));
                    }

                    let callee = self.stack.pop().expect("Missing callee");
                    self.stack.push(callee.apply(Value::Undefined, args)?);
                }
                _ => unimplemented!("{}", instruction),
            }
        }
    }
}
