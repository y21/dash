use std::ffi::{OsStr, OsString};
use std::fs::DirEntry;
use std::io;

// This is a list of tests that cannot be run currently, because they abort the process or run into an infinite loop
pub const IGNORED_TESTS: &[&str] = &[
    // very very large arrays
    "Array/prototype/indexOf/length-near-integer-limit.js",
    "Array/prototype/includes/length-boundaries.js",
    "Array/prototype/concat/arg-length-exceeding-integer-limit.js",
    "Array/prototype/fill/length-near-integer-limit.js",
    "Array/property-cast-number.js",
    "Array/prototype/unshift/clamps-to-integer-limit.js",
    "Array/prototype/unshift/throws-if-integer-limit-exceeded.js",
    "Array/prototype/unshift/clamps-to-integer-limit.js",
    "Array/prototype/unshift/length-near-integer-limit.js",
    "Array/prototype/map/15.4.4.19-3-29.js",
    "Array/prototype/map/15.4.4.19-3-14.js",
    "Array/prototype/map/15.4.4.19-3-8.js",
    "Array/prototype/map/15.4.4.19-3-28.js",
    "Array/prototype/slice/S15.4.4.10_A3_T2.js",
    "Array/prototype/slice/S15.4.4.10_A3_T1.js",
    "indexOf/15.4.4.14-9-9.js",
    "lastIndexOf/15.4.4.15-8-9.js",
    "push/S15.4.4.7_A3.js",
    "Array/S15.4.5.2_A3_T3.js",
    "Array/S15.4_A1.1_T10.js",
    "Array/S15.4.5.2_A1_T2.js",
    "Array/S15.4.5.2_A2_T1.js",
    "Array/S15.4.5.2_A3_T1.js",
    "Array/S15.4_A1.1_T5.js",
    "Array/S15.4.5.2_A3_T2.js",
    "Array/S15.4_A1.1_T4.js",
    "Array/S15.4_A1.1_T6.js",
    "Array/S15.4_A1.1_T7.js",
    "Array/S15.4_A1.1_T8.js",
    "Array/S15.4_A1.1_T9.js",
    "Array/S15.4.5.2_A1_T1.js",
    "Object/defineProperty/15.2.3.6-4-183.js",
    "Object/defineProperty/15.2.3.6-4-184.js",
    "Object/defineProperty/15.2.3.6-4-185.js",
    "Object/defineProperty/15.2.3.6-4-186.js",
    "Object/defineProperty/15.2.3.6-4-154.js",
    "Object/defineProperty/15.2.3.6-4-155.js",
    "Object/defineProperty/15.2.3.6-4-156.js",
    "Object/defineProperty/15.2.3.6-4-157.js",
    "Array/length/S15.4.2.2_A2.1_T1.js",
    "ArrayBuffer/allocation-limit.js",
    "ArrayBuffer/length-is-too-large-throws.js",
    // should throw a RangeError
    "create-non-array-invalid-len.js",
    // our `includes` currently ignores fromIndex so this runs into very long loops
    "Array/prototype/includes/length-boundaries.js",
    // Number.prototype.toString stack overflow
    "toString/S15.7.4.2_A4_T05.js",
    // interesting throw stack overflows
    "throw/S12.13_A3_T6.js",
    // infinite loops
    "try/S12.14_A9_T1.js",
    "while/S12.6.2_A9.js",
    "RegExp/S15.10.2_A1_T1.js",
    // `[^]+` causes an infinite loop during regex AnchorStart+Repetition
    "S15.10.2.13_A3_T3.js",
    "S15.10.2.13_A3_T4.js",
    "S15.10.2.13_A3_T5.js",
    "S15.10.2.13_A3_T6.js",
    "S15.10.2.13_A3_T7.js",
    "S15.10.2.13_A3_T8.js",
    "S15.10.2.13_A3_T9.js",
    "S15.10.2.7_A4_T1.js",
    "S15.10.2.7_A4_T2.js",
    "S15.10.2.7_A4_T3.js",
    "S15.10.2.7_A4_T4.js",
    "S15.10.2.7_A4_T5.js",
    "S15.10.2.7_A4_T6.js",
    "S15.10.2.7_A4_T7.js",
    "S15.10.2.7_A4_T8.js",
    "S15.10.2.7_A4_T9.js",
    "S15.10.2.13_A2_T4.js",
];

/// Returns a vector of path strings
pub fn get_all_files(dir: &OsStr) -> io::Result<Vec<OsString>> {
    let mut ve = Vec::new();

    let read_dir = std::fs::read_dir(dir)?;

    for entry in read_dir {
        let entry: DirEntry = entry?;

        let path = OsString::from(format!(
            "{}/{}",
            dir.to_str().unwrap(),
            entry.file_name().as_os_str().to_str().unwrap()
        ));

        if IGNORED_TESTS.iter().any(|t| path.to_str().unwrap().ends_with(t)) {
            continue;
        }

        let ty = entry.file_type()?;
        if ty.is_file() {
            ve.push(path);
        } else if ty.is_dir() {
            let files = get_all_files(&path)?;
            ve.extend(files);
        }
    }

    Ok(ve)
}
