use std::num::FpCategory;

use dash_middle::interner::{sym, Symbol};

use crate::localscope::LocalScope;

/// Marks the code path leading to this call as cold, or "unlikely"
#[cold]
pub fn cold_path() {}

/// Marks the boolean as unlikely to be true.
pub fn unlikely(b: bool) -> bool {
    if b {
        cold_path();
    }
    b
}

/// https://doc.rust-lang.org/beta/nightly-rustc/rustc_data_structures/captures/trait.Captures.html
/// and
/// https://github.com/rust-lang/rust/issues/34511#issuecomment-373423999
pub trait Captures<'a> {}

impl<'a, T: ?Sized> Captures<'a> for T {}

pub fn intern_f64(sc: &mut LocalScope, n: f64) -> Symbol {
    if n.trunc() == n && n >= 0.0 && n <= usize::MAX as f64 {
        // Happy path: no fractional part and fits in a usize
        // This can use the specialized usize interner
        return sc.intern_usize(n as usize);
    }

    match n.classify() {
        FpCategory::Infinite => sym::Infinity,
        FpCategory::Nan => sym::NaN,
        _ if n >= 1e21f64 || n <= -1e21f64 => {
            let mut digits = 0;
            let mut n = n;
            while n >= 10f64 {
                n /= 10f64;
                digits += 1;
            }
            while n <= -10f64 {
                n /= 10f64;
                digits += 1;
            }
            sc.intern(format!("{n:.0}e+{digits}").as_ref())
        }
        _ => sc.intern(n.to_string().as_ref()),
    }
}
