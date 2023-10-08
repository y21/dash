use std::any;
use std::path::Path;

use anyhow::Context;
use anyhow::{anyhow, bail};
use dash_middle::interner::StringInterner;
use dash_middle::parser::error::IntoFormattableErrors;
use dash_optimizer::OptLevel;
use dash_parser::Parser;
use dash_rt::format_value;
use dash_rt::runtime::Runtime;
use dash_vm::eval::EvalError;
use dash_vm::localscope::LocalScope;
use dash_vm::throw;
use dash_vm::value::function::native::CallContext;
use dash_vm::value::function::Function;
use dash_vm::value::function::FunctionKind;
use dash_vm::value::object::NamedObject;
use dash_vm::value::object::Object;
use dash_vm::value::object::PropertyValue;
use dash_vm::value::Value;
use package::Package;

mod package;

pub fn run_with_nodejs_mnemnoics(path: &str, opt: OptLevel, initial_gc_threshold: Option<usize>) -> anyhow::Result<()> {
    let tokio_rt = tokio::runtime::Runtime::new()?;

    tokio_rt.block_on(run_inner(path, opt, initial_gc_threshold));
    // let mut interner = StringInterner::default();
    // let mut parser = match Parser::new_from_str(&mut interner, code) {
    //     Ok(parser) => parser,
    //     Err(errs) => bail!("{}", errs.formattable(&interner, code, true)),
    // };
    // let (ast, counter) = match parser.parse_all() {
    //     Ok(ast) => ast,
    //     Err(errs) => bail!("{}", errs.formattable(&interner, code, true)),
    // };
    // ast.splice(0..0, [

    // ]);

    Ok(())
}

async fn run_inner(path: &str, opt: OptLevel, initial_gc_threshold: Option<usize>) {
    if let Err(err) = run_inner_fallible(path, opt, initial_gc_threshold).await {
        eprintln!("{}", err);
    }
}

async fn run_inner_fallible(path: &str, opt: OptLevel, initial_gc_threshold: Option<usize>) -> anyhow::Result<()> {
    let path = Path::new(path);
    if !path.is_dir() {
        bail!("Path needs to be a directory in node mode");
    }

    let package = std::fs::read_to_string(path.join("package.json")).context("Failed to read package.json")?;
    let package = serde_json::from_str::<Package>(&package)?;

    let entry = std::fs::read_to_string(path.join(package.main)).context("Failed to read entry point")?;

    let mut rt = Runtime::new(initial_gc_threshold).await;
    let scope = &mut rt.vm_mut().scope();
    let exports = Value::Object(scope.register(NamedObject::new(scope)));
    let module = Value::Object(scope.register(NamedObject::new(scope)));
    let require = Value::Object(scope.register(Function::new(
        scope,
        Some("require".into()),
        FunctionKind::Native(require),
    )));
    module
        .set_property(scope, "exports".into(), PropertyValue::static_default(exports.clone()))
        .unwrap();
    scope
        .global()
        .set_property(scope, "module".into(), PropertyValue::static_default(module))
        .unwrap();
    scope
        .global()
        .set_property(scope, "exports".into(), PropertyValue::static_default(exports))
        .unwrap();
    scope
        .global()
        .set_property(scope, "require".into(), PropertyValue::static_default(require))
        .unwrap();
    let value = scope.eval(&entry, opt).map_err(|(err, interner)| match err {
        EvalError::Middle(errs) => anyhow!("{}", errs.formattable(&interner, &entry, true)),
        EvalError::Exception(err) => anyhow!("{}", format_value(err.root(scope), scope).unwrap()),
    })?;

    Ok(())
}

fn require(cx: CallContext) -> Result<Value, Value> {
    let Some(Value::String(arg)) = cx.args.first() else {
        throw!(cx.scope, Error, "require() expects a string argument");
    };
    let is_path = matches!(arg.chars().next(), Some('.' | '/' | '~'));
    if is_path {
        todo!();
    } else {
        // Resolve dependency
        dbg!(std::env::current_dir());
    }
    Ok(Value::undefined())
}
