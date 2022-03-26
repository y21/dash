use std::rc::Rc;

use crate::throw;
use crate::vm::local::LocalScope;
use crate::vm::value::Value;

pub trait ValueConversion {
    fn to_primitive(
        &self,
        sc: &mut LocalScope,
        preferred_type: Option<PreferredType>,
    ) -> Result<Value, Value>;
    fn to_number(&self) -> Result<f64, Value>;
    fn to_boolean(&self) -> Result<bool, Value>;
    fn to_string(&self, sc: &mut LocalScope) -> Result<Rc<str>, Value>;
}

impl ValueConversion for Value {
    fn to_number(&self) -> Result<f64, Value> {
        match self {
            Value::Number(n) => Ok(*n),
            _ => todo!(), // TODO: implement other cases
        }
    }

    fn to_boolean(&self) -> Result<bool, Value> {
        match self {
            Value::Boolean(b) => Ok(*b),
            _ => todo!(), // TODO: implement other cases
        }
    }

    fn to_string(&self, sc: &mut LocalScope) -> Result<Rc<str>, Value> {
        match self {
            Value::String(s) => Ok(s.clone()),
            Value::Boolean(b) => Ok(b
                .then(|| sc.statics.get_true())
                .unwrap_or_else(|| sc.statics.get_false())),
            Value::Null(_) => Ok(sc.statics.null_str()),
            Value::Undefined(_) => Ok(sc.statics.undefined_str()),
            Value::Number(n) => Ok(n.to_string().into()), // TODO: we can do better
            Value::External(e) => {
                // ???
                todo!()
            }
            Value::Object(_) => {
                let prim_value = self.to_primitive(sc, Some(PreferredType::String))?;
                prim_value.to_string(sc)
            }
        }
    }

    fn to_primitive(
        &self,
        sc: &mut LocalScope,
        preferred_type: Option<PreferredType>,
    ) -> Result<Value, Value> {
        if let Value::Object(obj) = self {
            // TODO: Call @@toPrimitive instead of toString once we have symbols
            let exotic_to_prim = self.get_property(sc, "toString")?;

            if let Value::Undefined(_) = exotic_to_prim {
                // TODO: d. Return ? OrdinaryToPrimitive(input, preferredType).
                throw!(sc, "Failed to convert to primitive");
            }

            let result = exotic_to_prim.apply(sc, self.clone(), Vec::new())?;

            if let Value::Object(_) = result {
                throw!(sc, "Failed to convert to primitive");
            }

            Ok(result)
        } else {
            Ok(self.clone())
        }
    }
}

pub enum PreferredType {
    String,
    Number,
}
