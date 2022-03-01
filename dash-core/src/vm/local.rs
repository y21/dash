use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::gc::handle::Handle;

use super::value::object::Object;
use super::Vm;

#[derive(Debug)]
pub struct LocalScope<'a> {
    pub(crate) vm: &'a mut Vm,
}

impl<'a> LocalScope<'a> {
    pub fn new(vm: &'a mut Vm) -> Self {
        Self { vm }
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
