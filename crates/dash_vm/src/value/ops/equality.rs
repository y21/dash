use std::cmp::Ordering;

use crate::localscope::LocalScope;
use crate::value::object::Object as _;
use crate::value::{Unpack, Value, ValueKind};

use super::conversions::ValueConversion;

fn ord_value(left: Value, right: Value, sc: &mut LocalScope) -> Result<Option<Ordering>, Value> {
    if let (ValueKind::String(left), ValueKind::String(right)) = (left.unpack(), right.unpack()) {
        let left = left.res(sc);
        let right = right.res(sc);
        Ok(Some(left.cmp(right)))
    } else {
        let left = left.to_number(sc)?;
        let right = right.to_number(sc)?;
        Ok(left.partial_cmp(&right))
    }
}

pub fn lt(left: Value, right: Value, sc: &mut LocalScope) -> Result<bool, Value> {
    ord_value(left, right, sc).map(|ord| matches!(ord, Some(Ordering::Less)))
}

pub fn le(left: Value, right: Value, sc: &mut LocalScope) -> Result<bool, Value> {
    ord_value(left, right, sc).map(|ord| matches!(ord, Some(Ordering::Less | Ordering::Equal)))
}

pub fn gt(left: Value, right: Value, sc: &mut LocalScope) -> Result<bool, Value> {
    ord_value(left, right, sc).map(|ord| matches!(ord, Some(Ordering::Greater)))
}

pub fn ge(left: Value, right: Value, sc: &mut LocalScope) -> Result<bool, Value> {
    ord_value(left, right, sc).map(|ord| matches!(ord, Some(Ordering::Greater | Ordering::Equal)))
}

/// https://tc39.es/ecma262/multipage/abstract-operations.html#sec-islooselyequal
pub fn eq(left: Value, right: Value, sc: &mut LocalScope) -> Result<bool, Value> {
    // TODO: fast path for same tag..?
    if left.type_of(sc) == right.type_of(sc) {
        return Ok(strict_eq(left, right));
    }

    let left_full = left.unpack();
    let right_full = right.unpack();

    if let (ValueKind::Null(_), ValueKind::Undefined(_)) | (ValueKind::Undefined(_), ValueKind::Null(_)) =
        (left_full, right_full)
    {
        return Ok(true);
    }

    if let (ValueKind::Number(left), ValueKind::String(right)) = (left_full, right_full) {
        let num = right.to_number(sc)?;
        return Ok(left.0 == num);
    }

    if let (ValueKind::String(left), ValueKind::Number(right)) = (left_full, right_full) {
        let num = left.to_number(sc)?;
        return Ok(num == right.0);
    }

    if let ValueKind::Boolean(b) = left_full {
        return eq(Value::number(b.into()), right, sc);
    }

    if let ValueKind::Boolean(b) = right_full {
        return eq(left, Value::number(b.into()), sc);
    }

    if let (ValueKind::String(_) | ValueKind::Number(_) | ValueKind::Symbol(_), ValueKind::Object(_)) =
        (left_full, right_full)
    {
        let right = right.to_primitive(sc, None)?;
        return eq(left, right, sc);
    }

    if let (ValueKind::Object(_), ValueKind::String(_) | ValueKind::Number(_) | ValueKind::Symbol(_)) =
        (left_full, right_full)
    {
        let left = left.to_primitive(sc, None)?;
        return eq(left, right, sc);
    }

    Ok(false)
}

pub fn strict_eq(left: Value, right: Value) -> bool {
    match (left.unpack(), right.unpack()) {
        (ValueKind::Number(l), ValueKind::Number(r)) => l == r,
        (ValueKind::Boolean(l), ValueKind::Boolean(r)) => l == r,
        (ValueKind::String(l), ValueKind::String(r)) => l == r,
        (ValueKind::Undefined(_), ValueKind::Undefined(_)) => true,
        (ValueKind::Null(_), ValueKind::Null(_)) => true,
        (ValueKind::Symbol(l), ValueKind::Symbol(r)) => l == r,
        (ValueKind::Object(l), ValueKind::Object(r)) => l == r,
        (ValueKind::External(_), ValueKind::External(_)) => panic!("cannot compare external values"),
        _ => false,
    }
}

pub fn ne(left: Value, right: Value, sc: &mut LocalScope) -> Result<bool, Value> {
    eq(left, right, sc).map(|v| !v)
}

pub fn strict_ne(left: Value, right: Value) -> bool {
    !strict_eq(left, right)
}
