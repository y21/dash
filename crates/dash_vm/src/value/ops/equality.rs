use crate::localscope::LocalScope;
use crate::value::Value;

pub trait ValueEquality {
    fn lt(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value>;
    fn le(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value>;
    fn gt(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value>;
    fn ge(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value>;
    fn eq(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value>;
    fn strict_eq(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value>;
    fn ne(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        self.eq(other, sc).map(|v| v.not(sc))
    }
    fn strict_ne(&self, other: &Value, sc: &mut LocalScope) -> Result<Value, Value> {
        self.strict_eq(other, sc).map(|v| v.not(sc))
    }
}
