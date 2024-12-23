use std::cell::Cell;
use std::fmt;
use std::io::Read;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::RangeInclusive;
use std::thread::ThreadId;

const DIGIT: RangeInclusive<u8> = b'0'..=b'9';
const OCTAL_DIGIT: RangeInclusive<u8> = b'0'..=b'7';
const IDENTIFIER_START_LOWERCASE: RangeInclusive<u8> = b'a'..=b'z';
const IDENTIFIER_START_UPPERCASE: RangeInclusive<u8> = b'A'..=b'Z';
const HEX_LOWERCASE: RangeInclusive<u8> = b'a'..=b'f';
const HEX_UPPERCASE: RangeInclusive<u8> = b'A'..=b'F';

/// Checks whether the `c` is a digit
pub fn is_digit(c: u8) -> bool {
    DIGIT.contains(&c)
}

/// Checks whether `c` is a valid hex digit
pub fn is_hex_digit(c: u8) -> bool {
    DIGIT.contains(&c) || HEX_LOWERCASE.contains(&c) || HEX_UPPERCASE.contains(&c)
}

/// Checks whether `c` is a valid binary digit
pub fn is_binary_digit(c: u8) -> bool {
    c == b'0' || c == b'1'
}

/// Checks whether `c` is a valid octal digit
pub fn is_octal_digit(c: u8) -> bool {
    OCTAL_DIGIT.contains(&c)
}

/// Checks whether `c` is a valid identifier
pub fn is_alpha(c: u8) -> bool {
    is_identifier_start(c) || is_digit(c)
}

/// Checks if `c` is a valid character for the start of an identifier
pub fn is_identifier_start(c: u8) -> bool {
    IDENTIFIER_START_LOWERCASE.contains(&c)
        || IDENTIFIER_START_UPPERCASE.contains(&c)
        || c == b'_'
        || c == b'$'
        || c == b'#'
}

/// Checks if `c` is numeric
pub fn is_numeric(c: impl AsRef<str>) -> bool {
    c.as_ref().chars().all(|c| c.is_numeric())
}

pub fn next_char_in_bytes(b: &[u8]) -> (char, usize) {
    let byte_count = b.iter().take_while(|b| !b.is_ascii()).count();
    let bytes = &b[..byte_count];
    let s = force_utf8_borrowed(bytes);
    (s.chars().next().unwrap(), byte_count)
}

pub fn is_integer(n: f64) -> bool {
    n.fract() == 0.0
}

#[cold]
fn unlikely_inner() {}

pub fn unlikely(b: bool) -> bool {
    if b {
        unlikely_inner();
    }
    b
}

pub fn force_utf8(s: &[u8]) -> String {
    std::str::from_utf8(s).expect("Invalid UTF8").into()
}

pub fn force_utf8_borrowed(s: &[u8]) -> &str {
    std::str::from_utf8(s).expect("Invalid UTF8")
}

pub fn fmt_group<D: fmt::Display>(formatter: &mut fmt::Formatter<'_>, items: &[D], delim: &str) -> fmt::Result {
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            write!(formatter, "{delim}")?;
        }

        write!(formatter, "{item}")?;
    }
    Ok(())
}

pub struct Reader<R: Read>(R, usize);

impl<R: Read> Reader<R> {
    pub fn new(r: R) -> Self {
        Self(r, 0)
    }

    pub fn offset(&self) -> usize {
        self.1
    }

    pub fn read_bytes<const N: usize>(&mut self) -> Option<[u8; N]> {
        let mut buf = [0; N];
        self.0.read_exact(&mut buf).ok()?;
        self.1 += N;
        Some(buf)
    }

    pub fn read(&mut self) -> Option<u8> {
        self.read_bytes::<1>().map(|[b]| b)
    }

    pub fn read_u16_ne(&mut self) -> Option<u16> {
        self.read_bytes().map(u16::from_ne_bytes)
    }

    pub fn read_u32_ne(&mut self) -> Option<u32> {
        self.read_bytes().map(u32::from_ne_bytes)
    }

    pub fn read_i16_ne(&mut self) -> Option<i16> {
        self.read_bytes().map(i16::from_ne_bytes)
    }
}

/// A storage container for any value that is always `Send` and `Sync` regardless of its contents.
///
/// It does so soundly by only allowing access to the contained value on the same thread.
/// This allows moving `Value`s between threads (but not ever touching them), and eventually moving them back to the original thread.
///
/// Dropping the ThreadSafeValue on a different thread than it was created on will panic, and not drop the contained value.
//TODO: !! THIS SHOULD ABORT INSTEAD OF PANICKING FOR DROP !!
pub struct ThreadSafeStorage<T> {
    value: ManuallyDrop<T>,
    thread_id: ThreadId,
}

unsafe impl<T> Send for ThreadSafeStorage<T> {}
unsafe impl<T> Sync for ThreadSafeStorage<T> {}

impl<T> ThreadSafeStorage<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: ManuallyDrop::new(value),
            thread_id: std::thread::current().id(),
        }
    }

    pub fn get(&self) -> &T {
        self.assert_same_thread();
        &self.value
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.assert_same_thread();
        &mut self.value
    }

    fn assert_same_thread(&self) {
        assert_eq!(self.thread_id, std::thread::current().id());
    }
}

impl<T> Drop for ThreadSafeStorage<T> {
    fn drop(&mut self) {
        self.assert_same_thread();
        unsafe { ManuallyDrop::drop(&mut self.value) };
    }
}

/// A type that allows moving a value out of a shared reference (once).
pub struct SharedOnce<T>(Cell<Option<T>>);

impl<T> SharedOnce<T> {
    pub fn new(value: T) -> Self {
        Self(Cell::new(Some(value)))
    }

    pub fn take(&self) -> T {
        self.0.take().expect("Already taken")
    }

    pub fn try_take(&self) -> Option<T> {
        self.0.take()
    }
}
impl<T> fmt::Debug for SharedOnce<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedOnce").finish()
    }
}

#[derive(Debug, Default)]
pub struct LevelStack(Vec<u8>);

impl LevelStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_level(&mut self) {
        self.0.push(0);
    }

    pub fn inc_level(&mut self) -> Option<()> {
        *self.0.last_mut()? += 1;
        Some(())
    }

    pub fn dec_level(&mut self) -> Option<()> {
        *self.0.last_mut()? -= 1;
        Some(())
    }

    pub fn cur_level(&self) -> Option<u8> {
        self.0.last().copied()
    }

    pub fn pop_level(&mut self) -> Option<u8> {
        self.0.pop()
    }
}

#[macro_export]
macro_rules! timed {
    ($name:expr, $code:expr) => {{
        let start = std::time::Instant::now();
        let result = $code;
        let elapsed = start.elapsed();
        println!("{} - {:?}", $name, elapsed);
        result
    }};
}

#[derive(Debug, Clone, Copy)]
pub struct Counter<T>(usize, PhantomData<T>);

impl<T> Default for Counter<T> {
    fn default() -> Self {
        Self(0, PhantomData)
    }
}

impl<T> Counter<T>
where
    T: From<usize>,
{
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the highest ID that is currently in use
    pub fn highest(&self) -> Option<T> {
        if self.0 > 0 { Some(T::from(self.0 - 1)) } else { None }
    }

    pub fn len(&self) -> usize {
        self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn with(start: T) -> Self
    where
        T: Into<usize>,
    {
        Self(start.into(), PhantomData)
    }

    pub fn inc(&mut self) -> T {
        let old = self.0;
        self.0 += 1;
        T::from(old)
    }
}

#[macro_export]
macro_rules! if_match {
    ($scrutinee:expr => {
        $($sym:expr => $code:expr),+,
        $(_ => $fallback:expr)?
    }) => {{
        let scrutinee = $scrutinee;
        if false { loop {} }
        $(
            else if scrutinee == $sym {
                $code
            }
        )*
        $(
            else { $fallback }
        )?
    }};
}

#[macro_export]
macro_rules! with {
    ($v:expr, |$name:ident| $code:expr) => {{
        let $name = $v;
        $code
    }};
}
