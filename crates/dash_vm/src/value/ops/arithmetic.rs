use crate::localscope::LocalScope;
use crate::value::object::Object;
use crate::value::{Typeof, Value};

use super::abstractions::conversions::ValueConversion;
use super::equality::ValueEquality;

impl Value {
    pub fn add(&self, other: &Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let left = self.to_primitive(scope, None)?;
        let right = other.to_primitive(scope, None)?;

        let leftstr = matches!(left.type_of(), Typeof::String);
        let rightstr = matches!(right.type_of(), Typeof::String);

        if leftstr || rightstr {
            let lstr = left.to_string(scope)?;
            let rstr = right.to_string(scope)?;
            Ok(Value::String(format!("{lstr}{rstr}").into()))
        } else {
            let lnum = left.to_number(scope)?;
            let rnum = right.to_number(scope)?;
            Ok(Value::number(lnum + rnum))
        }
    }

    pub fn sub(&self, other: &Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let lnum = self.to_number(scope)?;
        let rnum = other.to_number(scope)?;
        Ok(Value::number(lnum - rnum))
    }

    pub fn mul(&self, other: &Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let lnum = self.to_number(scope)?;
        let rnum = other.to_number(scope)?;
        Ok(Value::number(lnum * rnum))
    }

    pub fn div(&self, other: &Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let lnum = self.to_number(scope)?;
        let rnum = other.to_number(scope)?;
        Ok(Value::number(lnum / rnum))
    }

    pub fn rem(&self, other: &Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let lnum = self.to_number(scope)?;
        let rnum = other.to_number(scope)?;
        Ok(Value::number(lnum % rnum))
    }

    pub fn pow(&self, other: &Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let lnum = self.to_number(scope)?;
        let rnum = other.to_number(scope)?;
        Ok(Value::number(lnum.powf(rnum)))
    }

    pub fn not(&self) -> Value {
        Value::Boolean(!self.is_truthy())
    }

    pub fn bitor(&self, other: &Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let this = self.to_int32(scope)?;
        let that = other.to_int32(scope)?;
        Ok(Value::number((this | that) as f64))
    }

    pub fn bitxor(&self, other: &Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let this = self.to_int32(scope)?;
        let that = other.to_int32(scope)?;
        Ok(Value::number((this ^ that) as f64))
    }

    pub fn bitand(&self, other: &Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let this = self.to_int32(scope)?;
        let that = other.to_int32(scope)?;
        Ok(Value::number((this & that) as f64))
    }

    pub fn bitshl(&self, other: &Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let this = self.to_int32(scope)?;
        let that = other.to_int32(scope)?;
        Ok(Value::number((this << that) as f64))
    }

    pub fn bitshr(&self, other: &Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let this = self.to_int32(scope)?;
        let that = other.to_int32(scope)?;
        Ok(Value::number((this >> that) as f64))
    }

    pub fn bitushr(&self, other: &Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let this = self.to_int32(scope)? as u32;
        let that = other.to_int32(scope)? as u32;
        Ok(Value::number((this.wrapping_shr(that)) as f64))
    }

    pub fn bitnot(&self, scope: &mut LocalScope) -> Result<Value, Value> {
        let this = self.to_int32(scope)?;
        Ok(Value::number((!this) as f64))
    }
}

macro_rules! delegate {
    ($self:expr, $other:expr, $sc:expr, $func:expr) => {
        match $self {
            Self::Number(n) => $func(n, $other, $sc),
            Self::Boolean(b) => $func(b, $other, $sc),
            Self::String(s) => $func(s, $other, $sc),
            Self::Undefined(u) => $func(u, $other, $sc),
            Self::Null(n) => $func(n, $other, $sc),
            Self::Symbol(s) => $func(s, $other, $sc),
            Self::Object(o) => {
                if let Some(prim) = o.as_primitive_capable() {
                    $func(prim, $other, $sc)
                } else {
                    Ok(Value::Boolean(match $other {
                        Self::Object(o2) => std::ptr::eq(o.as_ptr(), o2.as_ptr()),
                        Self::External(o2) => std::ptr::eq(o.as_ptr(), o2.inner.as_ptr()),
                        _ => false,
                    }))
                }
            }
            Self::External(o) => {
                if let Some(prim) = o.as_primitive_capable() {
                    $func(prim, $other, $sc)
                } else {
                    Ok(Value::Boolean(match $other {
                        Self::Object(o2) => std::ptr::eq(o.inner.as_ptr(), o2.as_ptr()),
                        Self::External(o2) => std::ptr::eq(o.inner.as_ptr(), o2.inner.as_ptr()),
                        _ => false,
                    }))
                }
            }
        }
    };
}

impl ValueEquality for Value {
    fn lt(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        delegate!(self, other, sc, ValueEquality::lt)
    }

    fn le(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        delegate!(self, other, sc, ValueEquality::le)
    }

    fn gt(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        delegate!(self, other, sc, ValueEquality::gt)
    }

    fn ge(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        delegate!(self, other, sc, ValueEquality::ge)
    }

    fn eq(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        delegate!(self, other, sc, ValueEquality::eq)
    }

    fn strict_eq(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        delegate!(self, other, sc, ValueEquality::strict_eq)
    }
}
