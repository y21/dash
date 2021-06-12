/// Same as std::result::Result, but repr(C)
#[repr(C)]
pub enum WasmResult<T, E> {
    Ok(T),
    Err(E)
}

impl<T, E> From<Result<T, E>> for WasmResult<T, E> {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(t) => Self::Ok(t),
            Err(e) => Self::Err(e)
        }
    }
}

impl<T, E> WasmResult<T, E> {
    pub fn as_result(&self) -> Result<&T, &E> {
        match self {
            Self::Ok(t) => Ok(t),
            Self::Err(e) => Err(e)
        }
    }
}

/// Same as std::option::Option, but repr(C)
#[repr(C)]
pub enum WasmOption<T> {
    Some(T),
    None
}

impl<T> From<Option<T>> for WasmOption<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(t) => Self::Some(t),
            _ => Self::None
        }
    }
}

impl<T> WasmOption<T> {
    pub fn as_option(&self) -> Option<&T> {
        match self {
            Self::Some(t) => Some(t),
            Self::None => None
        }
    }
}