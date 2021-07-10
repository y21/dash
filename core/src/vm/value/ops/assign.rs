use crate::vm::value::Value;

impl Value {
    /// Implements the addition assignment operator (lhs += rhs)
    pub fn add_assign(&mut self, other: &Value) {
        self.kind = self.add(other).kind;
    }

    /// Implements the subtraction assignment operator (lhs -= rhs)
    pub fn sub_assign(&mut self, other: &Value) {
        self.kind = self.sub(other).kind;
    }

    /// Implements the multiplication assignment operator (lhs *= rhs)
    pub fn mul_assign(&mut self, other: &Value) {
        self.kind = self.mul(other).kind;
    }

    /// Implements the division assignment operator (lhs /= rhs)
    pub fn div_assign(&mut self, other: &Value) {
        self.kind = self.div(other).kind;
    }

    /// Implements the remainder assignment operator (lhs %= rhs)
    pub fn rem_assign(&mut self, other: &Value) {
        self.kind = self.rem(other).kind;
    }

    /// Implements the exponentiation assignment operator (lhs **= rhs)
    pub fn pow_assign(&mut self, other: &Value) {
        self.kind = self.pow(other).kind;
    }

    /// Implements the left shift assignment operator (lhs <<= rhs)
    pub fn left_shift_assign(&mut self, other: &Value) {
        self.kind = self.left_shift(other).kind;
    }

    /// Implements the right shift assignment operator (lhs >>= rhs)
    pub fn right_shift_assign(&mut self, other: &Value) {
        self.kind = self.right_shift(other).kind;
    }

    /// Implements the unsigned right shift assignment operator (lhs >>>= rhs)
    pub fn unsigned_right_shift_assign(&mut self, other: &Value) {
        self.kind = self.unsigned_right_shift(other).kind;
    }

    /// Implements the bitwise and assignment operator (lhs &= rhs)
    pub fn bitwise_and_assign(&mut self, other: &Value) {
        self.kind = self.bitwise_and(other).kind;
    }

    /// Implements the bitwise or assignment operator (lhs |= rhs)
    pub fn bitwise_or_assign(&mut self, other: &Value) {
        self.kind = self.bitwise_or(other).kind;
    }

    /// Implements the bitwise xor assignment operator (lhs ^= rhs)
    pub fn bitwise_xor_assign(&mut self, other: &Value) {
        self.kind = self.bitwise_xor(other).kind;
    }

    /// Implements the logical and assignment operator (lhs &&= rhs)
    pub fn logical_and_assign(&mut self, other: &Value) {
        let re = self.logical_and_ref(other);
        if !std::ptr::eq(self, re) {
            self.kind = re.kind.clone();
        }
    }

    /// Implements the logical or assignment operator (lhs ||= rhs)
    pub fn logical_or_assign(&mut self, other: &Value) {
        let re = self.logical_or_ref(other);
        if !std::ptr::eq(self, re) {
            self.kind = re.kind.clone();
        }
    }

    /// Implements the nullish coalescing assignment operator (lhs ??= rhs)
    pub fn nullish_coalescing_assign(&mut self, other: &Value) {
        let re = self.nullish_coalescing_ref(other);
        if !std::ptr::eq(self, re) {
            self.kind = re.kind.clone();
        }
    }
}
