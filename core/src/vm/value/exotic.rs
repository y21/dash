use std::{borrow::Cow, fmt::Debug};

use crate::{gc::Handle, js_std, vm::VM};

use super::Value;

fn not_implemented_error(vm: &VM) -> Handle<Value> {
    js_std::error::create_error("not yet implemented".into(), vm)
}

/// Clonable exotic object
pub trait ExoticClone {
    /// Clones this Exotic trait object
    fn clone_box(&self) -> Box<dyn Exotic>;
}

impl<T> ExoticClone for T
where
    T: Exotic + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn Exotic> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Exotic> {
    fn clone(&self) -> Box<dyn Exotic> {
        self.clone_box()
    }
}

/// Implements object behavior
pub trait Exotic: Debug + ExoticClone {
    /// Inspects a JavaScript value
    fn inspect(&self, this: &Value, depth: u32) -> Cow<str>;

    /// Trap for function calls
    fn apply(
        &self,
        _this: Handle<VM>,
        _args: Vec<Handle<Value>>,
        vm: &mut VM,
    ) -> Result<Handle<Value>, Handle<Value>> {
        Err(not_implemented_error(vm))
    }

    /// Trap for the `new` operator
    fn construct(&self, vm: &VM) -> Result<Handle<Value>, Handle<Value>> {
        Err(not_implemented_error(vm))
    }

    /// Trap for defining a new property
    fn define_property(&mut self, vm: &VM) -> Result<Handle<Value>, Handle<Value>> {
        Err(not_implemented_error(vm))
    }

    /// Trap for deleting a property
    fn delete_property(&mut self, vm: &VM) -> Result<Handle<Value>, Handle<Value>> {
        Err(not_implemented_error(vm))
    }
    /// Trap for getting a property
    fn get(&self, vm: &VM) -> Result<Handle<Value>, Handle<Value>> {
        Err(not_implemented_error(vm))
    }

    /// Trap for Object.getOwnPropertyDescriptor
    fn get_own_property_descriptor(&self, vm: &VM) -> Result<Handle<Value>, Handle<Value>> {
        Err(not_implemented_error(vm))
    }

    /// Trap for the [[GetPrototypeOf]] internal method
    fn get_prototype_of(&self, vm: &VM) -> Result<Handle<Value>, Handle<Value>> {
        Err(not_implemented_error(vm))
    }

    /// Trap for the `in` operator
    fn has(&self, vm: &VM) -> Result<Handle<Value>, Handle<Value>> {
        Err(not_implemented_error(vm))
    }

    /// Trap for Object.isExtensible
    fn is_extensible(&self, vm: &VM) -> Result<Handle<Value>, Handle<Value>> {
        Err(not_implemented_error(vm))
    }

    /// Trap for Reflect.ownKeys
    fn own_keys(&self, vm: &VM) -> Result<Handle<Value>, Handle<Value>> {
        Err(not_implemented_error(vm))
    }

    /// Trap for Object.preventExtensions
    fn prevent_extensions(&mut self, vm: &VM) -> Result<Handle<Value>, Handle<Value>> {
        Err(not_implemented_error(vm))
    }

    /// Trap for the `set` operator
    fn set(&mut self, vm: &VM) -> Result<Handle<Value>, Handle<Value>> {
        Err(not_implemented_error(vm))
    }

    /// Trap for Object.setPrototypeOf
    fn set_prototype_of(&mut self, vm: &VM) -> Result<Handle<Value>, Handle<Value>> {
        Err(not_implemented_error(vm))
    }
}
