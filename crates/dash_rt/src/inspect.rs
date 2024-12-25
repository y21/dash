use std::fmt::Write;

use dash_middle::interner::{Symbol, sym};

use dash_vm::frame::This;
use dash_vm::localscope::LocalScope;
use dash_vm::util::intern_f64;
use dash_vm::value::array::{Array, ArrayIterator};
use dash_vm::value::arraybuffer::ArrayBuffer;
use dash_vm::value::error::Error;
use dash_vm::value::typedarray::TypedArray;
use dash_vm::value::{Typeof, Unpack, ValueKind};

use dash_vm::value::object::{Object, PropertyDataDescriptor, PropertyKey, PropertyValueKind};
use dash_vm::value::ops::conversions::ValueConversion;
use dash_vm::value::primitive::Number;
use dash_vm::value::root_ext::RootErrExt;
use dash_vm::value::{Root, Value};

#[derive(Copy, Clone)]
pub struct InspectOptions {
    /// Whether to invoke any getters
    invoke_getters: bool,
    /// The max depth
    depth: u32,
    /// Whether to use colors
    colors: bool,
}
impl Default for InspectOptions {
    fn default() -> Self {
        Self {
            invoke_getters: false,
            depth: 8,
            colors: true,
        }
    }
}

const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const GREY: &str = "\x1b[38;5;m";
const RESET: &str = "\x1b[0m";

fn colored<R>(s: &mut String, options: InspectOptions, color: &str, f: impl FnOnce(&mut String) -> R) -> R {
    if options.colors {
        *s += color;
        let res = f(s);
        *s += RESET;
        res
    } else {
        f(s)
    }
}

fn debug_inspect_string(s: &str) -> String {
    format!("{s:?}")
}

fn inspect_array_into(
    array: Value,
    scope: &mut LocalScope<'_>,
    options: InspectOptions,
    depth: u32,
    out: &mut String,
) -> Result<(), Value> {
    let iterator = ArrayIterator::new(scope, array)?;
    if iterator.is_empty() {
        *out += "[]";
        return Ok(());
    }

    *out += "[ ";
    let mut count = 0;
    while let Some(value) = iterator.next(scope).root(scope)? {
        if count > 0 {
            *out += ", ";
        }
        inspect_inner_into(value, scope, options, depth + 1, out)?;
        count += 1;
    }
    *out += " ]";
    Ok(())
}

fn inspect_arraybuffer_into(arraybuffer: &ArrayBuffer, constructor: Symbol, out: &mut String) {
    write!(out, "{constructor}({}) {{ ", arraybuffer.len()).unwrap();
    for (i, byte) in arraybuffer.storage().iter().enumerate().take(32) {
        if i > 0 {
            *out += " ";
        }
        write!(out, "{:02x}", byte.get()).unwrap();
    }
    if arraybuffer.len() > 32 {
        *out += " ...";
    }
    *out += " }";
}

fn inspect_inner_into(
    value: Value,
    scope: &mut LocalScope<'_>,
    options: InspectOptions,
    depth: u32,
    out: &mut String,
) -> Result<(), Value> {
    if depth > options.depth {
        *out += "...";
        return Ok(());
    } else if depth > 5000 {
        *out += "/* recursion limit reached */";
        return Ok(());
    }

    match value.unpack() {
        ValueKind::String(string) => {
            if depth > 1 {
                colored(out, options, GREEN, |s| {
                    *s += &*debug_inspect_string(string.res(scope));
                })
            } else {
                *out += string.res(scope)
            }
        }
        ValueKind::Number(Number(number)) => colored(out, options, YELLOW, |s| {
            let sym = intern_f64(scope, number);
            *s += scope.interner.resolve(sym);
        }),
        ValueKind::Boolean(boolean) => colored(out, options, YELLOW, |s| *s += if boolean { "true" } else { "false" }),
        ValueKind::Undefined(_) => colored(out, options, GREY, |s| *s += "undefined"),
        ValueKind::Null(_) => colored(out, options, GREY, |s| *s += "null"),
        ValueKind::Symbol(symbol) => colored(out, options, YELLOW, |s| {
            *s += &*("@@".to_owned() + scope.interner.resolve(symbol.sym()));
        }),
        ValueKind::Object(object) => {
            let constructor = object.get_property(scope, sym::constructor.into()).root(scope)?;
            let constructor_name = constructor
                .get_property(scope, sym::name.into())
                .root(scope)?
                .to_js_string(scope)?;

            if object.extract::<Array>(scope).is_some() {
                return inspect_array_into(value, scope, options, depth, out);
            }

            if let Some(error) = object.extract::<Error>(scope) {
                *out += error.stack.res(scope);
                return Ok(());
            }

            // ArrayBuffer and views
            if let Some(arraybuffer) = object
                .extract::<ArrayBuffer>(scope)
                .or_else(|| object.extract::<TypedArray>(scope).map(|t| t.arraybuffer(scope)))
            {
                inspect_arraybuffer_into(arraybuffer, constructor_name.sym(), out);
                return Ok(());
            }

            if object.type_of(scope) == Typeof::Function {
                let name = object
                    .get_own_property(scope, sym::name.into())
                    .root(scope)?
                    .into_option()
                    .map(|v| v.to_js_string(scope))
                    .transpose()?
                    .map(|s| s.res(scope))
                    .filter(|v| !v.is_empty())
                    .unwrap_or("(anonymous)");

                write!(out, "[Function: {name}]").unwrap();
                return Ok(());
            }

            if constructor != Value::object(scope.statics.object_ctor) {
                *out += constructor_name.res(scope);
                *out += " ";
            }

            *out += "{ ";
            let keys = object.own_keys(scope)?;
            for (i, key) in keys.into_iter().enumerate() {
                let key = PropertyKey::from_value(scope, key)?;

                if let Some(property_value) = object.get_own_property_descriptor(scope, key).root_err(scope)? {
                    if property_value.descriptor.contains(PropertyDataDescriptor::ENUMERABLE) {
                        if i > 0 {
                            *out += ", ";
                        }

                        match key {
                            PropertyKey::String(string) => {
                                let string = string.res(scope);
                                if string.bytes().any(|v| {
                                    dash_middle::util::is_identifier_start(v) || dash_middle::util::is_alpha(v)
                                }) {
                                    *out += string;
                                } else {
                                    colored(out, options, GREEN, |s| *s += &*debug_inspect_string(string));
                                }
                            }
                            PropertyKey::Symbol(symbol) => {
                                *out += "[";
                                inspect_inner_into(Value::symbol(symbol), scope, options, depth + 1, out)?;
                                *out += "]";
                            }
                        };

                        *out += ": ";

                        match property_value.kind {
                            PropertyValueKind::Trap { .. } => {
                                if options.invoke_getters {
                                    colored(out, options, GREY, |s| *s += "(computed) ");
                                    inspect_inner_into(
                                        property_value.get_or_apply(scope, This::Bound(value)).root(scope)?,
                                        scope,
                                        options,
                                        depth + 1,
                                        out,
                                    )?;
                                } else {
                                    colored(out, options, GREY, |s| *s += "(trap)");
                                }
                            }
                            PropertyValueKind::Static(value) => {
                                inspect_inner_into(value, scope, options, depth + 1, out)?;
                            }
                        }
                    }
                }
            }
            *out += " }";
        }
        ValueKind::External(_) => unreachable!(),
    }

    Ok(())
}

/// Deeply "inspects" (debug formats) a JavaScript value.
pub fn inspect(value: Value, scope: &mut LocalScope<'_>, options: InspectOptions) -> Result<String, Value> {
    let mut out = String::new();
    inspect_inner_into(value, scope, options, 1, &mut out).map(|_| out)
}
