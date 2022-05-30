/// Adds two values together
pub const ADD: u8 = 0x01;
pub const SUB: u8 = 0x02;
pub const MUL: u8 = 0x03;
pub const DIV: u8 = 0x04;
pub const REM: u8 = 0x05;
pub const POW: u8 = 0x06;
pub const GT: u8 = 0x07;
pub const GE: u8 = 0x08;
pub const LT: u8 = 0x09;
pub const LE: u8 = 0x0A;
pub const EQ: u8 = 0x0B;
pub const NE: u8 = 0x0C;
/// Discards the last value on the stack
pub const POP: u8 = 0x0D;
/// Loads a local value
pub const LDLOCAL: u8 = 0x0E;
pub const LDLOCALW: u8 = 0x0F;
pub const LDGLOBAL: u8 = 0x10;
pub const LDGLOBALW: u8 = 0x11;
pub const CONSTANT: u8 = 0x12;
pub const CONSTANTW: u8 = 0x13;
pub const POS: u8 = 0x14;
/// Negates the last value on the stack
pub const NEG: u8 = 0x15;
pub const TYPEOF: u8 = 0x16;
pub const BITNOT: u8 = 0x17;
pub const NOT: u8 = 0x18;
pub const STORELOCAL: u8 = 0x19;
pub const STORELOCALW: u8 = 0x1A;
pub const STOREGLOBAL: u8 = 0x1B;
pub const STOREGLOBALW: u8 = 0x1C;
pub const RET: u8 = 0x1D;
pub const CALL: u8 = 0x1E;
/// Jumps to the given label
pub const JMPFALSEP: u8 = 0x1F;
pub const JMP: u8 = 0x21;
pub const STATICPROPACCESS: u8 = 0x23;
pub const STATICPROPACCESSW: u8 = 0x24;
pub const DYNAMICPROPACCESS: u8 = 0x25;
pub const ARRAYLIT: u8 = 0x26;
pub const ARRAYLITW: u8 = 0x27;
pub const OBJLIT: u8 = 0x28;
pub const OBJLITW: u8 = 0x29;
pub const THIS: u8 = 0x2A;
pub const STATICPROPSET: u8 = 0x2B;
pub const STATICPROPSETW: u8 = 0x2C;
pub const DYNAMICPROPSET: u8 = 0x2D;
/// Loads an "extern" local variable, existing in a parent scope
pub const LDLOCALEXT: u8 = 0x2E;
pub const LDLOCALEXTW: u8 = 0x2F;
pub const STORELOCALEXT: u8 = 0x30;
pub const STORELOCALEXTW: u8 = 0x31;
pub const STRICTEQ: u8 = 0x32;
pub const STRICTNE: u8 = 0x33;
pub const TRY: u8 = 0x34;
pub const TRYEND: u8 = 0x35;
pub const THROW: u8 = 0x36;
pub const YIELD: u8 = 0x37;
/// Jumps to a given label if the last value on the stack is false, but does **not** actually pop the value
pub const JMPFALSENP: u8 = 0x38;
pub const JMPTRUEP: u8 = 0x39;
pub const JMPTRUENP: u8 = 0x3A;
pub const JMPNULLISHP: u8 = 0x3B;
pub const JMPNULLISHNP: u8 = 0x3C;
pub const BITOR: u8 = 0x3D;
pub const BITXOR: u8 = 0x3E;
pub const BITAND: u8 = 0x3F;
pub const BITSHL: u8 = 0x40;
pub const BITSHR: u8 = 0x41;
pub const BITUSHR: u8 = 0x42;
pub const OBJIN: u8 = 0x43;
pub const INSTANCEOF: u8 = 0x44;
/// ImportKind::Dynamic
pub const IMPORTDYN: u8 = 0x45;
/// ImportKind::DefaultAs
/// ImportKind::AllAs
pub const IMPORTSTATIC: u8 = 0x46;
pub const EXPORTDEFAULT: u8 = 0x47;
pub const EXPORTNAMED: u8 = 0x48;
pub const DEBUGGER: u8 = 0x49;
pub const GLOBAL: u8 = 0x4A;
pub const SUPER: u8 = 0x4B;
/// "Reverses" the last N stack values
/// (e.g. REVSTACK 3: `[0, 1, 2, 3]` becomes `[0, 3, 2, 1]`)
pub const REVSTCK: u8 = 0x4C;
