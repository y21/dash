use crate::vm::instruction::Constant;

/// A pool of constants, used by the compiler to store compile-time values
#[derive(Debug, Clone)]
pub struct ConstantPool(Vec<Constant>);

impl ConstantPool {
    /// Creates a new constant pool
    pub fn new() -> Self {
        ConstantPool(Vec::new())
    }

    /// Adds a constant to the inner constant pool and returns its index
    pub fn add(&mut self, constant: Constant) -> u8 {
        let len = self.0.len();
        assert!(len < u8::MAX as usize);
        self.0.push(constant);
        len as u8
    }

    /// Returns the constant at the given index
    pub fn get(&self, index: u8) -> Option<&Constant> {
        self.0.get(index as usize)
    }

    /// Boxes the inner vector of constants and returns it
    pub fn into_boxed_slice(self) -> Box<[Constant]> {
        self.0.into_boxed_slice()
    }
}

impl From<ConstantPool> for Box<[Constant]> {
    fn from(this: ConstantPool) -> Self {
        this.into_boxed_slice()
    }
}

impl From<ConstantPool> for Vec<Constant> {
    fn from(this: ConstantPool) -> Self {
        this.0
    }
}
