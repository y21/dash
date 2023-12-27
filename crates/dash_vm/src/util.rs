use std::num::FpCategory;

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

pub fn format_f64(n: f64) -> String {
    // TODO: specialize zero, infinity, NaN by "interning" them in vm.statics
    match n.classify() {
        FpCategory::Infinite => "Infinity".into(),
        FpCategory::Nan => "NaN".into(),
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
            format!("{n:.0}e+{digits}")
        }
        _ => format!("{n}"),
    }
}
