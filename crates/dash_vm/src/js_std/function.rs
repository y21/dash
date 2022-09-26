use std::borrow::Cow;
use std::rc::Rc;

use dash_compiler::FunctionCompiler;
use dash_middle::compiler::constant::Function as MiddleFunction;
use dash_middle::parser::statement::VariableBinding;
use dash_middle::parser::statement::VariableDeclarationKind;

use crate::throw;
use crate::value::function::bound::BoundFunction;
use crate::value::function::native::CallContext;
use crate::value::function::user::UserFunction;
use crate::value::function::Function;
use crate::value::function::FunctionKind;
use crate::value::ops::abstractions::conversions::ValueConversion;
use crate::value::Typeof;
use crate::value::Value;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let code = cx
        .args
        .last()
        .map(|v| v.to_string(cx.scope))
        .transpose()?
        .unwrap_or_else(|| cx.scope.statics().empty_str());

    let parameters = cx
        .args
        .iter()
        .take(cx.args.len().saturating_sub(1))
        .map(|x| x.to_string(cx.scope))
        .collect::<Result<Vec<_>, _>>()?;

    let opt = cx.scope.params().opt_level().unwrap_or_default();

    let fun = {
        let mut compiler = FunctionCompiler::new(opt);
        let scope = compiler.scope_mut();
        for parameter in parameters {
            let result = scope.add_local(
                VariableBinding {
                    kind: VariableDeclarationKind::Var,
                    name: Cow::Owned(ToString::to_string(&parameter)),
                    ty: None,
                },
                false,
            );

            if let Err(..) = result {
                throw!(cx.scope, "Too many function parameters");
            }
        }

        let cr = match compiler.compile_str(&code) {
            Ok(cr) => cr,
            Err(err) => throw!(cx.scope, "{:?}", err),
        };

        let middle_fun = MiddleFunction::from_compile_result(cr);
        let fun = UserFunction::new(Rc::new(middle_fun), Rc::new([]));
        Function::new(cx.scope, None, FunctionKind::User(fun))
    };
    let fun = cx.scope.register(fun);
    Ok(Value::Object(fun))
}

pub fn bind(cx: CallContext) -> Result<Value, Value> {
    let target_this = cx.args.first().cloned();
    let target_args = cx.args.get(1..).map(|s| s.to_vec());
    let target_callee = match cx.this {
        Value::Object(o) if matches!(o.type_of(), Typeof::Function) => o,
        _ => throw!(cx.scope, "Bound value must be a function"),
    };

    let bf = BoundFunction::new(cx.scope, target_callee, target_this, target_args);
    Ok(Value::Object(cx.scope.register(bf)))
}

pub fn call(cx: CallContext) -> Result<Value, Value> {
    let target_this = cx.args.first().cloned();
    let target_args = cx.args.get(1..).map(|s| s.to_vec());
    let target_callee = match cx.this {
        Value::Object(o) if matches!(o.type_of(), Typeof::Function) => o,
        _ => throw!(cx.scope, "Bound value must be a function"),
    };

    target_callee.apply(
        cx.scope,
        target_this.unwrap_or_else(|| Value::undefined()),
        target_args.unwrap_or_default(),
    )
}
