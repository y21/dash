use std::{cell::RefCell, rc::Rc};

use super::error::{self, MaybeRc};
use crate::vm::value::{function::CallContext, Value};

pub fn abs(ctx: CallContext) -> Result<Rc<RefCell<Value>>, Rc<RefCell<Value>>> {
    let num = ctx
        .args
        .first()
        .map(|c| c.borrow().as_number())
        .unwrap_or(f64::NAN);

    Ok(ctx.vm.create_js_value(num.abs()).into())
}

pub fn ceil(ctx: CallContext) -> Result<Rc<RefCell<Value>>, Rc<RefCell<Value>>> {
    let num = ctx
        .args
        .first()
        .map(|c| c.borrow().as_number())
        .unwrap_or(f64::NAN);

    Ok(ctx.vm.create_js_value(num.ceil()).into())
}

pub fn floor(ctx: CallContext) -> Result<Rc<RefCell<Value>>, Rc<RefCell<Value>>> {
    let num = ctx
        .args
        .first()
        .map(|c| c.borrow().as_number())
        .unwrap_or(f64::NAN);

    Ok(ctx.vm.create_js_value(num.floor()).into())
}

pub fn max(ctx: CallContext) -> Result<Rc<RefCell<Value>>, Rc<RefCell<Value>>> {
    let mut arguments = ctx.arguments();
    let mut max = match arguments.next().cloned() {
        Some(value) => value,
        None => return Ok(ctx.vm.create_js_value(-f64::INFINITY).into()),
    };
    let mut max_num = max.borrow().as_number();

    for arg_cell in arguments {
        let arg = arg_cell.borrow().as_number();
        if arg > max_num {
            max_num = arg;
            max = Rc::clone(&arg_cell);
        }
    }

    Ok(max)
}

pub fn min(ctx: CallContext) -> Result<Rc<RefCell<Value>>, Rc<RefCell<Value>>> {
    let mut arguments = ctx.arguments();
    let mut max = match arguments.next().cloned() {
        Some(value) => value,
        None => return Ok(ctx.vm.create_js_value(f64::INFINITY).into()),
    };
    let mut max_num = max.borrow().as_number();

    for arg_cell in arguments {
        let arg = arg_cell.borrow().as_number();
        if arg < max_num {
            max_num = arg;
            max = Rc::clone(&arg_cell);
        }
    }

    Ok(max)
}

pub fn pow(ctx: CallContext) -> Result<Rc<RefCell<Value>>, Rc<RefCell<Value>>> {
    let mut args = ctx.arguments();

    let lhs = args
        .next()
        .map(|n| n.borrow().as_number())
        .unwrap_or(f64::NAN);

    let rhs = args
        .next()
        .map(|n| n.borrow().as_number())
        .unwrap_or(f64::NAN);

    let result = lhs.powf(rhs);

    Ok(ctx.vm.create_js_value(result).into())
}

pub fn random(ctx: CallContext) -> Result<Rc<RefCell<Value>>, Rc<RefCell<Value>>> {
    let maybe_random = ctx.vm.agent.random();

    match maybe_random {
        Some(rand) => Ok(ctx.vm.create_js_value(rand).into()),
        None => Err(error::create_error(
            MaybeRc::Owned("Random number generation failed"),
            ctx.vm,
        )),
    }
}
