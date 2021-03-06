use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::gc::handle::Handle;

use super::value::object::Object;
use super::value::Value;
use super::Vm;

#[derive(Debug)]
pub struct LocalScope<'a> {
    pub(crate) vm: &'a mut Vm,
}

impl<'a> LocalScope<'a> {
    pub fn new(vm: &'a mut Vm) -> Self {
        Self { vm }
    }

    pub fn add_ref(&mut self, obj: Handle<dyn Object>) {
        let this = self as *const LocalScope;
        self.vm.externals.add_single(this, obj);
    }

    pub fn add_value(&mut self, value: Value) {
        if let Value::External(o) | Value::Object(o) = value {
            self.add_ref(o);
        }
    }

    pub fn register<O: Object + 'static>(&mut self, obj: O) -> Handle<dyn Object> {
        let handle = self.vm.register(obj);
        self.add_ref(handle.clone());
        handle
    }
}

impl<'a> Drop for LocalScope<'a> {
    fn drop(&mut self) {
        let this = self as *const LocalScope;
        self.vm.externals.remove(this);
    }
}

impl<'a> Deref for LocalScope<'a> {
    type Target = Vm;

    fn deref(&self) -> &Self::Target {
        self.vm
    }
}

impl<'a> DerefMut for LocalScope<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.vm
    }
}

pub struct Local<'s>(Handle<dyn Object>, PhantomData<&'s ()>);
