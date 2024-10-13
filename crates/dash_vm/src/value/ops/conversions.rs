use crate::gc::interner::sym;
use crate::gc::ObjectId;
use crate::localscope::LocalScope;
use crate::throw;
use crate::value::boxed::{Boolean, Number as BoxedNumber, String as BoxedString, Symbol as BoxedSymbol};
use crate::value::object::Object;
use crate::value::primitive::{Number, MAX_SAFE_INTEGERF};
use crate::value::string::JsString;
use crate::value::{Root, Typeof, Unpack, Value, ValueKind};

pub trait ValueConversion {
    fn to_primitive(&self, sc: &mut LocalScope, preferred_type: Option<PreferredType>) -> Result<Value, Value>;

    fn to_number(&self, sc: &mut LocalScope) -> Result<f64, Value>;

    fn to_length(&self, sc: &mut LocalScope) -> Result<f64, Value> {
        // Let len be ? ToIntegerOrInfinity(argument).
        let len = self.to_integer_or_infinity(sc)?;
        // 2. If len ≤ 0, return +0𝔽.
        if len <= 0.0 {
            return Ok(0.0);
        }

        // Return 𝔽(min(len, 253 - 1)).
        Ok(len.min(MAX_SAFE_INTEGERF))
    }

    fn to_length_u(&self, sc: &mut LocalScope) -> Result<usize, Value> {
        self.to_length(sc).map(|x| x as usize)
    }

    fn to_integer_or_infinity(&self, sc: &mut LocalScope) -> Result<f64, Value> {
        // Let number be ? ToNumber(argument).
        let number = self.to_number(sc)?;
        // 2. If number is NaN, +0𝔽, or -0𝔽, return 0.
        if number.is_nan() || number == 0.0 {
            return Ok(0.0);
        }

        // 3. If number is +∞𝔽, return +∞.
        // 4. If number is -∞𝔽, return -∞.
        if number.is_infinite() {
            return Ok(number);
        }

        // 5. Let integer be floor(abs(ℝ(number))).
        let integer = number.abs().floor();

        // 6. If number < -0𝔽, set integer to -integer.
        if number < 0.0 { Ok(-integer) } else { Ok(integer) }
    }

    fn to_boolean(&self, sc: &mut LocalScope<'_>) -> Result<bool, Value>;

    fn to_js_string(&self, sc: &mut LocalScope) -> Result<JsString, Value>;

    fn length_of_array_like(&self, sc: &mut LocalScope) -> Result<usize, Value>;

    fn to_object(&self, sc: &mut LocalScope) -> Result<ObjectId, Value>;

    fn to_int32(&self, sc: &mut LocalScope) -> Result<i32, Value> {
        let n = self.to_number(sc)?;
        Ok(n as i64 as i32)
    }
}

impl ValueConversion for Value {
    fn to_number(&self, sc: &mut LocalScope) -> Result<f64, Value> {
        match self.unpack() {
            ValueKind::Number(Number(n)) => Ok(n),
            ValueKind::Undefined(_) => Ok(f64::NAN),
            ValueKind::Null(_) => Ok(0.0),
            ValueKind::Boolean(b) => Ok(b as i8 as f64),
            ValueKind::String(s) => match s.len(sc) {
                0 => Ok(0.0),
                _ => Ok(s.res(sc).parse::<f64>().unwrap_or(f64::NAN)),
            },
            ValueKind::Symbol(_) => throw!(sc, TypeError, "Cannot convert symbol to number"),
            ValueKind::Object(_) => self.to_primitive(sc, Some(PreferredType::Number))?.to_number(sc),
            ValueKind::External(_) => unreachable!(),
        }
    }

    fn to_boolean(&self, sc: &mut LocalScope<'_>) -> Result<bool, Value> {
        match self.unpack() {
            ValueKind::Boolean(b) => Ok(b),
            ValueKind::Undefined(_) => Ok(false),
            ValueKind::Null(_) => Ok(false),
            ValueKind::Number(Number(n)) => Ok(n != 0.0 && !n.is_nan()),
            ValueKind::String(s) => Ok(!s.res(sc).is_empty()),
            ValueKind::Symbol(_) => Ok(true),
            ValueKind::Object(_) => Ok(true),
            ValueKind::External(_) => unreachable!(),
        }
    }

    fn to_js_string(&self, sc: &mut LocalScope) -> Result<JsString, Value> {
        match self.unpack() {
            ValueKind::String(s) => ValueConversion::to_js_string(&s, sc),
            ValueKind::Boolean(b) => ValueConversion::to_js_string(&b, sc),
            ValueKind::Null(n) => ValueConversion::to_js_string(&n, sc),
            ValueKind::Undefined(u) => ValueConversion::to_js_string(&u, sc),
            ValueKind::Number(n) => ValueConversion::to_js_string(&n, sc),
            ValueKind::Object(_) => self.to_primitive(sc, Some(PreferredType::String))?.to_js_string(sc),
            ValueKind::Symbol(_) => throw!(sc, TypeError, "Cannot convert symbol to a string"),
            ValueKind::External(_) => unreachable!(),
        }
    }

    fn to_primitive(&self, sc: &mut LocalScope, preferred_type: Option<PreferredType>) -> Result<Value, Value> {
        // 1. If Type(input) is Object, then
        // (If not, return as is)
        if !matches!(self.unpack(), ValueKind::Object(_)) {
            return Ok(self.clone());
        }

        // a. Let exoticToPrim be ? GetMethod(input, @@toPrimitive).
        let to_primitive = sc.statics.symbol_to_primitive.clone();
        let exotic_to_prim = self.get_property(sc, to_primitive.into()).root(sc)?.into_option();

        // b. If exoticToPrim is not undefined, then
        if let Some(exotic_to_prim) = exotic_to_prim {
            // i. If preferredType is not present, let hint be "default".
            let preferred_type = preferred_type.unwrap_or(PreferredType::Default);

            let preferred_type = preferred_type.to_value();

            // iv. Let result be ? Call(exoticToPrim, input, « hint »).
            let result = exotic_to_prim.apply(sc, self.clone(), vec![preferred_type]).root(sc)?;

            // If Type(result) is not Object, return result.
            // TODO: this can still be an object if Value::External
            // TODO2: ^ can it? we usually unbox all locals on use, so you can't return an external
            if !matches!(result.unpack(), ValueKind::Object(_)) {
                return Ok(result);
            }

            // vi. Throw a TypeError exception.
            throw!(sc, TypeError, "Failed to convert to primitive");
        }

        // i. If preferredType is not present, let hint be "default".
        let preferred_type = preferred_type.unwrap_or(PreferredType::Number);

        self.ordinary_to_primitive(sc, preferred_type)
    }

    fn length_of_array_like(&self, sc: &mut LocalScope) -> Result<usize, Value> {
        self.get_property(sc, sym::length.into()).root(sc)?.to_length_u(sc)
    }

    fn to_object(&self, sc: &mut LocalScope) -> Result<ObjectId, Value> {
        fn register_dyn<O: Object + 'static, F: Fn(&mut LocalScope) -> O>(
            sc: &mut LocalScope,
            fun: F,
        ) -> Result<ObjectId, Value> {
            let obj = fun(sc);
            Ok(sc.register(obj))
        }

        match self.unpack() {
            ValueKind::Object(o) => Ok(o),
            ValueKind::Undefined(_) => throw!(sc, TypeError, "Cannot convert undefined to object"),
            ValueKind::Null(_) => throw!(sc, TypeError, "Cannot convert null to object"),
            ValueKind::Boolean(b) => register_dyn(sc, |sc| Boolean::new(sc, b)),
            ValueKind::Symbol(s) => register_dyn(sc, |sc| BoxedSymbol::new(sc, s)),
            ValueKind::Number(Number(n)) => register_dyn(sc, |sc| BoxedNumber::new(sc, n)),
            ValueKind::String(s) => register_dyn(sc, |sc| BoxedString::new(sc, s)),
            ValueKind::External(_) => unreachable!(),
        }
    }
}

pub enum PreferredType {
    Default,
    String,
    Number,
}

impl PreferredType {
    pub fn to_value(&self) -> Value {
        Value::string(match self {
            PreferredType::Default => sym::default.into(),
            PreferredType::String => sym::string.into(),
            PreferredType::Number => sym::number.into(),
        })
    }
}

impl Value {
    pub fn ordinary_to_primitive(&self, sc: &mut LocalScope, preferred_type: PreferredType) -> Result<Value, Value> {
        let method_names = match preferred_type {
            PreferredType::String => [sym::toString, sym::valueOf],
            PreferredType::Number | PreferredType::Default => [sym::valueOf, sym::toString],
        };

        for name in method_names {
            let method = self.get_property(sc, name.into()).root(sc)?;
            if matches!(method.type_of(sc), Typeof::Function) {
                let this = self.clone();
                let result = method.apply(sc, this, Vec::new()).root(sc)?;
                if !matches!(result.unpack(), ValueKind::Object(_)) {
                    return Ok(result);
                }
            }
        }

        throw!(sc, TypeError, "Failed to convert to primitive")
    }
}
