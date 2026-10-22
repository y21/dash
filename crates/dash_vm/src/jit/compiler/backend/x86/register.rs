#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[expect(dead_code)]
pub enum Register {
    Al,
    Ah,
    Ax,
    Eax,
    Rax,

    Cl,
    Ch,
    Cx,
    Ecx,
    Rcx,

    Dl,
    Dh,
    Dx,
    Edx,
    Rdx,

    Bl,
    Bh,
    Bx,
    Ebx,
    Rbx,

    Spl,
    Sp,
    Esp,
    Rsp,

    Bpl,
    Bp,
    Ebp,
    Rbp,

    Sil,
    Si,
    Esi,
    Rsi,

    Dil,
    Di,
    Edi,
    Rdi,

    R8b,
    R8w,
    R8d,
    R8,

    R9b,
    R9w,
    R9d,
    R9,

    R10b,
    R10w,
    R10d,
    R10,

    R11b,
    R11w,
    R11d,
    R11,

    R12b,
    R12w,
    R12d,
    R12,

    R13b,
    R13w,
    R13d,
    R13,

    R14b,
    R14w,
    R14d,
    R14,

    R15b,
    R15w,
    R15d,
    R15,

    Xmm0,
    Xmm1,
    Xmm2,
    Xmm3,
    Xmm4,
    Xmm5,
    Xmm6,
    Xmm7,
    Xmm8,
    Xmm9,
    Xmm10,
    Xmm11,
    Xmm12,
    Xmm13,
    Xmm14,
    Xmm15,
}

impl Register {
    pub fn size_bits(self) -> u32 {
        match self {
            // Register::Rax
            // | Register::R8
            // | Register::Rbp
            // | Register::R12
            // | Register::R13
            // | Register::Rsp
            // | Register::Rdi
            // | Register::Rsi
            // | Register::R14
            // | Register::R15
            // | Register::Rdx
            // | Register::Rcx
            // | Register::Rbx
            // | Register::R9
            // | Register::R10
            // | Register::R11
            // | Register::Xmm0
            // | Register::Xmm1
            // | Register::Xmm2
            // | Register::Xmm3
            // | Register::Xmm4
            // | Register::Xmm5
            // | Register::Xmm6
            // | Register::Xmm7
            // | Register::Xmm8
            // | Register::Xmm9
            // | Register::Xmm10
            // | Register::Xmm11
            // | Register::Xmm12
            // | Register::Xmm13
            // | Register::Xmm14
            // | Register::Xmm15 => 64,
            // Register::Eax => 32,
            // Register::Al => 8,
            Register::Al
            | Register::Bl
            | Register::Cl
            | Register::Dl
            | Register::Ah
            | Register::Bh
            | Register::Ch
            | Register::Dh
            | Register::Spl
            | Register::Bpl
            | Register::Sil
            | Register::Dil
            | Register::R8b
            | Register::R9b
            | Register::R10b
            | Register::R11b
            | Register::R12b
            | Register::R13b
            | Register::R14b
            | Register::R15b => 8,

            Register::Ax
            | Register::Bx
            | Register::Cx
            | Register::Dx
            | Register::Sp
            | Register::Bp
            | Register::Si
            | Register::Di
            | Register::R8w
            | Register::R9w
            | Register::R10w
            | Register::R11w
            | Register::R12w
            | Register::R13w
            | Register::R14w
            | Register::R15w => 16,

            Register::Eax
            | Register::Ebx
            | Register::Ecx
            | Register::Edx
            | Register::Esp
            | Register::Ebp
            | Register::Esi
            | Register::Edi
            | Register::R8d
            | Register::R9d
            | Register::R10d
            | Register::R11d
            | Register::R12d
            | Register::R13d
            | Register::R14d
            | Register::R15d => 32,

            Register::Rax
            | Register::Rbx
            | Register::Rcx
            | Register::Rdx
            | Register::Rsp
            | Register::Rbp
            | Register::Rsi
            | Register::Rdi
            | Register::R8
            | Register::R9
            | Register::R10
            | Register::R11
            | Register::R12
            | Register::R13
            | Register::R14
            | Register::R15 => 64,

            Register::Xmm0
            | Register::Xmm1
            | Register::Xmm2
            | Register::Xmm3
            | Register::Xmm4
            | Register::Xmm5
            | Register::Xmm6
            | Register::Xmm7
            | Register::Xmm8
            | Register::Xmm9
            | Register::Xmm10
            | Register::Xmm11
            | Register::Xmm12
            | Register::Xmm13
            | Register::Xmm14
            | Register::Xmm15 => 128,
        }
    }

    /// Require that the size of the register is the same as the size of the other register, and return it.
    pub fn join_register_size(self, other: Register) -> u32 {
        assert_eq!(
            self.size_bits(),
            other.size_bits(),
            "Register {self:?} of size {} does not match register {other:?} of size {}",
            self.size_bits(),
            other.size_bits()
        );
        self.size_bits()
    }

    /// Returns the register field and a boolean indicating if it requires a REX prefix.
    pub fn fields(self) -> u8 {
        match self {
            Register::Al | Register::Ax | Register::Eax | Register::Rax | Register::Xmm0 => 0,
            Register::Cl | Register::Cx | Register::Ecx | Register::Rcx | Register::Xmm1 => 1,
            Register::Dl | Register::Dx | Register::Edx | Register::Rdx | Register::Xmm2 => 2,
            Register::Bl | Register::Bx | Register::Ebx | Register::Rbx | Register::Xmm3 => 3,
            Register::Ah | Register::Spl | Register::Sp | Register::Esp | Register::Rsp | Register::Xmm4 => 4,
            Register::Ch | Register::Bpl | Register::Bp | Register::Ebp | Register::Rbp | Register::Xmm5 => 5,
            Register::Dh | Register::Sil | Register::Si | Register::Esi | Register::Rsi | Register::Xmm6 => 6,
            Register::Bh | Register::Dil | Register::Di | Register::Edi | Register::Rdi | Register::Xmm7 => 7,

            Register::R8b | Register::R8w | Register::R8d | Register::R8 | Register::Xmm8 => 0,
            Register::R9b | Register::R9w | Register::R9d | Register::R9 | Register::Xmm9 => 1,
            Register::R10b | Register::R10w | Register::R10d | Register::R10 | Register::Xmm10 => 2,
            Register::R11b | Register::R11w | Register::R11d | Register::R11 | Register::Xmm11 => 3,
            Register::R12b | Register::R12w | Register::R12d | Register::R12 | Register::Xmm12 => 4,
            Register::R13b | Register::R13w | Register::R13d | Register::R13 | Register::Xmm13 => 5,
            Register::R14b | Register::R14w | Register::R14d | Register::R14 | Register::Xmm14 => 6,
            Register::R15b | Register::R15w | Register::R15d | Register::R15 | Register::Xmm15 => 7,
        }
    }

    #[rustfmt::skip]
    pub fn needs_rex_prefix(self) -> bool {
        match self {
            Register::Al | Register::Ax | Register::Eax | Register::Rax | Register::Xmm0 |
            Register::Cl | Register::Cx | Register::Ecx | Register::Rcx | Register::Xmm1 |
            Register::Dl | Register::Dx | Register::Edx | Register::Rdx | Register::Xmm2 |
            Register::Bl | Register::Bx | Register::Ebx | Register::Rbx | Register::Xmm3 |
            Register::Ah | Register::Spl | Register::Sp | Register::Esp | Register::Rsp | Register::Xmm4 |
            Register::Ch | Register::Bpl | Register::Bp | Register::Ebp | Register::Rbp | Register::Xmm5 |
            Register::Dh | Register::Sil | Register::Si | Register::Esi | Register::Rsi | Register::Xmm6 |
            Register::Bh | Register::Dil | Register::Di | Register::Edi | Register::Rdi | Register::Xmm7 => false,

            Register::R8b | Register::R8w | Register::R8d | Register::R8 | Register::Xmm8 |
            Register::R9b | Register::R9w | Register::R9d | Register::R9 | Register::Xmm9 |
            Register::R10b | Register::R10w | Register::R10d | Register::R10 | Register::Xmm10 |
            Register::R11b | Register::R11w | Register::R11d | Register::R11 | Register::Xmm11 |
            Register::R12b | Register::R12w | Register::R12d | Register::R12 | Register::Xmm12 |
            Register::R13b | Register::R13w | Register::R13d | Register::R13 | Register::Xmm13 |
            Register::R14b | Register::R14w | Register::R14d | Register::R14 | Register::Xmm14 |
            Register::R15b | Register::R15w | Register::R15d | Register::R15 | Register::Xmm15 => true,
        }
    }

    pub const RM_FIELD_NEEDS_SIB: u8 = 0b100;
    pub fn rm_field(self) -> u8 {
        match self {
            Register::Al
            | Register::Ax
            | Register::Eax
            | Register::Rax
            | Register::Xmm0
            | Register::R8
            | Register::R8b
            | Register::R8w
            | Register::R8d
            | Register::Xmm8 => 0b000,
            Register::Cl
            | Register::Cx
            | Register::Ecx
            | Register::Rcx
            | Register::Xmm1
            | Register::R9
            | Register::R9b
            | Register::R9w
            | Register::R9d
            | Register::Xmm9 => 0b001,
            Register::Dl
            | Register::Dx
            | Register::Edx
            | Register::Rdx
            | Register::Xmm2
            | Register::R10
            | Register::R10b
            | Register::R10w
            | Register::R10d
            | Register::Xmm10 => 0b010,
            Register::Bl
            | Register::Bx
            | Register::Ebx
            | Register::Rbx
            | Register::Xmm3
            | Register::R11
            | Register::R11b
            | Register::R11w
            | Register::R11d
            | Register::Xmm11 => 0b011,
            Register::Ah
            | Register::Spl
            | Register::Sp
            | Register::Esp
            | Register::Rsp
            | Register::Xmm4
            | Register::R12
            | Register::R12b
            | Register::R12w
            | Register::R12d
            | Register::Xmm12 => 0b100,
            Register::Ch
            | Register::Bpl
            | Register::Bp
            | Register::Ebp
            | Register::Rbp
            | Register::Xmm5
            | Register::R13
            | Register::R13b
            | Register::R13w
            | Register::R13d
            | Register::Xmm13 => 0b101,
            Register::Dh
            | Register::Sil
            | Register::Si
            | Register::Esi
            | Register::Rsi
            | Register::Xmm6
            | Register::R14
            | Register::R14b
            | Register::R14w
            | Register::R14d
            | Register::Xmm14 => 0b110,
            Register::Bh
            | Register::Dil
            | Register::Di
            | Register::Edi
            | Register::Rdi
            | Register::Xmm7
            | Register::R15
            | Register::R15b
            | Register::R15w
            | Register::R15d
            | Register::Xmm15 => 0b111,
        }
    }

    pub fn to_32(&self) -> Register {
        match self {
            Register::Rax | Register::Eax | Register::Ax | Register::Al => Register::Eax,
            Register::Rbx | Register::Ebx | Register::Bx | Register::Bl => Register::Ebx,
            Register::Rcx | Register::Ecx | Register::Cx | Register::Cl => Register::Ecx,
            Register::Rdx | Register::Edx | Register::Dx | Register::Dl => Register::Edx,
            Register::Rsi | Register::Esi | Register::Si | Register::Sil => Register::Esi,
            Register::Rdi | Register::Edi | Register::Di | Register::Dil => Register::Edi,
            Register::Rbp | Register::Ebp | Register::Bp | Register::Bpl => Register::Ebp,
            Register::Rsp | Register::Esp | Register::Sp | Register::Spl => Register::Esp,
            Register::R8 | Register::R8d | Register::R8w | Register::R8b => Register::R8d,
            Register::R9 | Register::R9d | Register::R9w | Register::R9b => Register::R9d,
            Register::R10 | Register::R10d | Register::R10w | Register::R10b => Register::R10d,
            Register::R11 | Register::R11d | Register::R11w | Register::R11b => Register::R11d,
            Register::R12 | Register::R12d | Register::R12w | Register::R12b => Register::R12d,
            Register::R13 | Register::R13d | Register::R13w | Register::R13b => Register::R13d,
            Register::R14 | Register::R14d | Register::R14w | Register::R14b => Register::R14d,
            Register::R15 | Register::R15d | Register::R15w | Register::R15b => Register::R15d,
            _ => panic!("Cannot convert {:?} to 32-bit register", self),
        }
    }

    pub fn to_8(&self) -> Register {
        match self {
            Register::Rax | Register::Eax | Register::Ax | Register::Al => Register::Al,
            Register::Rbx | Register::Ebx | Register::Bx | Register::Bl => Register::Bl,
            Register::Rcx | Register::Ecx | Register::Cx | Register::Cl => Register::Cl,
            Register::Rdx | Register::Edx | Register::Dx | Register::Dl => Register::Dl,
            Register::Rsi | Register::Esi | Register::Si | Register::Sil => Register::Sil,
            Register::Rdi | Register::Edi | Register::Di | Register::Dil => Register::Dil,
            Register::Rbp | Register::Ebp | Register::Bp | Register::Bpl => Register::Bpl,
            Register::Rsp | Register::Esp | Register::Sp | Register::Spl => Register::Spl,
            Register::R8 | Register::R8d | Register::R8w | Register::R8b => Register::R8b,
            Register::R9 | Register::R9d | Register::R9w | Register::R9b => Register::R9b,
            Register::R10 | Register::R10d | Register::R10w | Register::R10b => Register::R10b,
            Register::R11 | Register::R11d | Register::R11w | Register::R11b => Register::R11b,
            Register::R12 | Register::R12d | Register::R12w | Register::R12b => Register::R12b,
            Register::R13 | Register::R13d | Register::R13w | Register::R13b => Register::R13b,
            Register::R14 | Register::R14d | Register::R14w | Register::R14b => Register::R14b,
            Register::R15 | Register::R15d | Register::R15w | Register::R15b => Register::R15b,
            _ => panic!("Cannot convert {:?} to 8-bit register", self),
        }
    }
}

pub fn usable_callee_saved_regs() -> Vec<Register> {
    // Intentionally in reverse order such that pop() will yield regs that don't need a REX prefix first
    vec![
        Register::R15,
        Register::R14,
        Register::R13,
        Register::R12,
        Register::Rbx,
    ]
}
pub fn usable_caller_saved_gprs() -> Vec<Register> {
    // Intentionally in reverse order such that pop() will yield regs that don't need a REX prefix first
    vec![
        Register::R11,
        Register::R10,
        Register::R9,
        Register::R8,
        Register::Rdi,
        Register::Rsi,
        Register::Rdx,
        Register::Rcx,
        Register::Rax,
    ]
}
pub fn usable_fprs() -> Vec<Register> {
    // Intentionally in reverse order such that pop() will yield regs that don't need a REX prefix first
    vec![
        Register::Xmm15,
        Register::Xmm14,
        Register::Xmm13,
        Register::Xmm12,
        Register::Xmm11,
        Register::Xmm10,
        Register::Xmm9,
        Register::Xmm8,
        Register::Xmm7,
        Register::Xmm6,
        Register::Xmm5,
        Register::Xmm4,
        Register::Xmm3,
        Register::Xmm2,
        Register::Xmm1,
        Register::Xmm0,
    ]
}
