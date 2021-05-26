use crate::js_std;

use super::{object::PropertyLookup, ValueKind};

pub fn get_property_unboxed(inner: &str, k: &str) -> Option<PropertyLookup> {
    match k {
        "length" => Some(PropertyLookup::Value(ValueKind::Number(inner.len() as f64))),
        "indexOf" => Some(PropertyLookup::Function(
            js_std::string::index_of,
            "indexOf",
            false,
        )),
        _ => None,
    }
}
