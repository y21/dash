use crate::localscope::LocalScope;
use crate::value::object::Object as _;
use crate::value::{Typeof, Value};

use super::conversions::ValueConversion;

impl Value {
    pub fn add(self, other: Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let left = self.to_primitive(scope, None)?;
        let right = other.to_primitive(scope, None)?;

        let leftstr = matches!(left.type_of(scope), Typeof::String);
        let rightstr = matches!(right.type_of(scope), Typeof::String);

        if leftstr || rightstr {
            let lstr = left.to_js_string(scope)?;
            let rstr = right.to_js_string(scope)?;
            let out = format!("{}{}", lstr.res(scope), rstr.res(scope));
            Ok(Value::string(scope.intern(out).into()))
        } else {
            let lnum = left.to_number(scope)?;
            let rnum = right.to_number(scope)?;
            Ok(Value::number(lnum + rnum))
        }
    }

    pub fn sub(self, other: Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let lnum = self.to_number(scope)?;
        let rnum = other.to_number(scope)?;
        Ok(Value::number(lnum - rnum))
    }

    pub fn mul(self, other: Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let lnum = self.to_number(scope)?;
        let rnum = other.to_number(scope)?;
        Ok(Value::number(lnum * rnum))
    }

    pub fn div(self, other: Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let lnum = self.to_number(scope)?;
        let rnum = other.to_number(scope)?;
        Ok(Value::number(lnum / rnum))
    }

    pub fn rem(self, other: Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let lnum = self.to_number(scope)?;
        let rnum = other.to_number(scope)?;
        Ok(Value::number(lnum % rnum))
    }

    pub fn pow(self, other: Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let lnum = self.to_number(scope)?;
        let rnum = other.to_number(scope)?;
        Ok(Value::number(lnum.powf(rnum)))
    }

    pub fn not(self, sc: &mut LocalScope<'_>) -> Value {
        Value::boolean(!self.is_truthy(sc))
    }

    pub fn bitor(self, other: Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let this = self.to_int32(scope)?;
        let that = other.to_int32(scope)?;
        Ok(Value::number((this | that) as f64))
    }

    pub fn bitxor(self, other: Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let this = self.to_int32(scope)?;
        let that = other.to_int32(scope)?;
        Ok(Value::number((this ^ that) as f64))
    }

    pub fn bitand(self, other: Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let this = self.to_int32(scope)?;
        let that = other.to_int32(scope)?;
        Ok(Value::number((this & that) as f64))
    }

    pub fn bitshl(self, other: Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let this = self.to_int32(scope)?;
        let that = other.to_int32(scope)?;
        Ok(Value::number((this << that) as f64))
    }

    pub fn bitshr(self, other: Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let this = self.to_int32(scope)?;
        let that = other.to_int32(scope)?;
        Ok(Value::number((this >> that) as f64))
    }

    pub fn bitushr(self, other: Self, scope: &mut LocalScope) -> Result<Value, Value> {
        let this = self.to_int32(scope)? as u32;
        let that = other.to_int32(scope)? as u32;
        Ok(Value::number((this.wrapping_shr(that)) as f64))
    }

    pub fn bitnot(self, scope: &mut LocalScope) -> Result<Value, Value> {
        let this = self.to_int32(scope)?;
        Ok(Value::number((!this) as f64))
    }
}
