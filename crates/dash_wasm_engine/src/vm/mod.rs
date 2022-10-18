pub mod value;

use std::rc::Rc;

use crate::parser::Program;

use self::value::Value;

pub struct Frame {
    buf: Rc<[u8]>,
    pc: usize,
}

pub struct Vm {
    stack: Vec<Value>,
    frames: Vec<Frame>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            frames: Vec::new(),
        }
    }

    fn fetch_and_inc_ip(&mut self) -> u8 {
        let pc = self.frames.last_mut().unwrap().pc;
        let byte = self.frames.last().unwrap().buf[pc];
        self.frames.last_mut().unwrap().pc += 1;
        byte
    }

    pub fn run(&mut self) {
        loop {
            let opcode = self.fetch_and_inc_ip();
            const LOCALGET: u8 = 0x20;
            match opcode {
                LOCALGET => {
                    let index = self.fetch_and_inc_ip();
                    let value = self.stack[index as usize].clone();
                    self.stack.push(value);
                }
                _ => panic!("unknown opcode: {}", opcode),
            }
        }
    }
}
