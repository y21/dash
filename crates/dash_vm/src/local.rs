use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::gc2::handle::Handle;

use super::value::object::Object;
use super::value::Value;
use super::Vm;

#[derive(Debug)]
pub struct LocalScope<'a> {
    // This lets us hold multiple LocalScopes of the same Vm unsafely without UB
    vm: *mut Vm,
    _p: PhantomData<&'a mut Vm>,
}

impl<'a> LocalScope<'a> {
    pub fn new(vm: &'a mut Vm) -> Self {
        Self { vm, _p: PhantomData }
    }

    pub fn add_ref(&mut self, obj: Handle<dyn Object>) {
        let this = self as *const LocalScope;
        self.externals.add_single(this, obj);
    }

    pub fn add_value(&mut self, value: Value) {
        match value {
            Value::Object(o) => self.add_ref(o),
            Value::External(o) => self.add_ref(o.inner.clone()),
            _ => {}
        }
    }

    pub fn register<O: Object + 'static>(&mut self, obj: O) -> Handle<dyn Object> {
        let handle = self.deref_mut().register(obj);
        self.add_ref(handle.clone());
        handle
    }
}

impl<'a> Drop for LocalScope<'a> {
    fn drop(&mut self) {
        let this = self as *const LocalScope;
        self.externals.remove(this);
    }
}

impl<'a> Deref for LocalScope<'a> {
    type Target = Vm;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.vm }
    }
}

impl<'a> DerefMut for LocalScope<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.vm }
    }
}

pub struct Local<'s>(Handle<dyn Object>, PhantomData<&'s ()>);
