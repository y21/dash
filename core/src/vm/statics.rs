use std::{cell::RefCell, mem::MaybeUninit, rc::Rc};

use crate::js_std;

use super::value::{NativeFunction, Value};

pub mod id {
    pub const CONSOLE_LOG: usize = 0;
    pub const ISNAN: usize = 1;
    pub const MATH_POW: usize = 2;
}

const ID_COUNT: usize = 3;

/// Static values
pub struct Statics([MaybeUninit<Rc<RefCell<Value>>>; ID_COUNT]);

impl Statics {
    pub fn new() -> Self {
        let mut statics = unsafe { Self(MaybeUninit::uninit().assume_init()) };
        statics.prepare();
        statics
    }

    fn prepare(&mut self) {
        self.set(
            id::ISNAN,
            NativeFunction::new("isNaN", js_std::functions::is_nan, None).into(),
        );

        self.set(
            id::MATH_POW,
            NativeFunction::new("pow", js_std::math::pow, None).into(),
        );

        self.set(
            id::CONSOLE_LOG,
            NativeFunction::new("log", js_std::console::log, None).into(),
        );
    }

    pub fn set(&mut self, id: usize, value: Rc<RefCell<Value>>) {
        self.0[id] = MaybeUninit::new(value);
    }

    pub unsafe fn get_unchecked(&self, id: usize) -> &Rc<RefCell<Value>> {
        &*self.0[id].as_ptr()
    }

    pub unsafe fn get_mut_unchecked(&mut self, id: usize) -> &mut Rc<RefCell<Value>> {
        &mut *self.0[id].as_mut_ptr()
    }

    pub unsafe fn iter_unchecked(&self) -> StaticsIter<'_> {
        StaticsIter(self, 0)
    }
}

impl Drop for Statics {
    fn drop(&mut self) {
        for idx in 0..ID_COUNT {
            unsafe {
                std::ptr::drop_in_place(self.0[idx].as_mut_ptr());
            }
        }
    }
}

pub struct StaticsIter<'a>(&'a Statics, usize);
impl<'a> Iterator for StaticsIter<'a> {
    type Item = Rc<RefCell<Value>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.1 >= ID_COUNT {
            return None;
        }

        let value = unsafe { self.0.get_unchecked(self.1) };

        self.1 += 1;

        Some(value.clone())
    }
}
