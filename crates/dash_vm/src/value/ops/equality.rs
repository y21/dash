use std::cmp::Ordering;

use crate::localscope::LocalScope;
use crate::value::Value;

use super::conversions::ValueConversion;

fn ord_value(left: &Value, right: &Value, sc: &mut LocalScope) -> Result<Option<Ordering>, Value> {
    if let (Value::String(left), Value::String(right)) = (left, right) {
        let left = left.res(sc);
        let right = right.res(sc);
        Ok(Some(left.cmp(right)))
    } else {
        let left = left.to_number(sc)?;
        let right = right.to_number(sc)?;
        Ok(left.partial_cmp(&right))
    }
}

pub fn lt(left: &Value, right: &Value, sc: &mut LocalScope) -> Result<bool, Value> {
    ord_value(left, right, sc).map(|ord| matches!(ord, Some(Ordering::Less)))
}

pub fn le(left: &Value, right: &Value, sc: &mut LocalScope) -> Result<bool, Value> {
    ord_value(left, right, sc).map(|ord| matches!(ord, Some(Ordering::Less | Ordering::Equal)))
}

pub fn gt(left: &Value, right: &Value, sc: &mut LocalScope) -> Result<bool, Value> {
    ord_value(left, right, sc).map(|ord| matches!(ord, Some(Ordering::Greater)))
}

pub fn ge(left: &Value, right: &Value, sc: &mut LocalScope) -> Result<bool, Value> {
    ord_value(left, right, sc).map(|ord| matches!(ord, Some(Ordering::Greater | Ordering::Equal)))
}

/// https://tc39.es/ecma262/multipage/abstract-operations.html#sec-islooselyequal
pub fn eq(left: &Value, right: &Value, sc: &mut LocalScope) -> Result<bool, Value> {
    if left.type_of() == right.type_of() {
        return Ok(strict_eq(left, right));
    }

    if let (Value::Null(_), Value::Undefined(_)) | (Value::Undefined(_), Value::Null(_)) = (left, right) {
        return Ok(true);
    }

    if let (Value::Number(left), Value::String(right)) = (left, right) {
        let num = right.to_number(sc)?;
        return Ok(left.0 == num);
    }

    if let (Value::String(left), Value::Number(right)) = (left, right) {
        let num = left.to_number(sc)?;
        return Ok(num == right.0);
    }

    if let &Value::Boolean(b) = left {
        return eq(&Value::Number(b.into()), right, sc);
    }

    if let &Value::Boolean(b) = right {
        return eq(left, &Value::Number(b.into()), sc);
    }

    if let (Value::String(_) | Value::Number(_) | Value::Symbol(_), Value::Object(_)) = (left, right) {
        let right = right.to_primitive(sc, None)?;
        return eq(left, &right, sc);
    }

    if let (Value::Object(_), Value::String(_) | Value::Number(_) | Value::Symbol(_)) = (left, right) {
        let left = left.to_primitive(sc, None)?;
        return eq(&left, right, sc);
    }

    Ok(false)
}

pub fn strict_eq(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(l), Value::Number(r)) => l == r,
        (Value::Boolean(l), Value::Boolean(r)) => l == r,
        (Value::String(l), Value::String(r)) => l == r,
        (Value::Undefined(_), Value::Undefined(_)) => true,
        (Value::Null(_), Value::Null(_)) => true,
        (Value::Symbol(l), Value::Symbol(r)) => l == r,
        (Value::Object(l), Value::Object(r)) => l == r,
        (Value::External(_), Value::External(_)) => panic!("cannot compare external values"),
        _ => false,
    }
}

pub fn ne(left: &Value, right: &Value, sc: &mut LocalScope) -> Result<bool, Value> {
    eq(left, right, sc).map(|v| !v)
}

pub fn strict_ne(left: &Value, right: &Value) -> bool {
    !strict_eq(left, right)
}
