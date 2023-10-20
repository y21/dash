use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::Context;
use anyhow::{anyhow, bail};
use dash_middle::interner::StringInterner;
use dash_middle::parser::error::IntoFormattableErrors;
use dash_optimizer::OptLevel;
use dash_proc_macro::Trace;
use dash_rt::format_value;
use dash_rt::runtime::Runtime;
use dash_vm::eval::EvalError;
use dash_vm::localscope::LocalScope;
use dash_vm::value::object::NamedObject;
use dash_vm::value::object::Object;
use dash_vm::value::object::PropertyValue;
use dash_vm::value::Value;
use dash_vm::{delegate, throw};
use package::Package;
use rustc_hash::FxHashMap;

mod package;

pub fn run_with_nodejs_mnemnoics(path: &str, opt: OptLevel, initial_gc_threshold: Option<usize>) -> anyhow::Result<()> {
    let tokio_rt = tokio::runtime::Runtime::new()?;

    tokio_rt.block_on(async move {
        if let Err(err) = run_inner_fallible(path, opt, initial_gc_threshold).await {
            eprintln!("{}", err);
        }
    });

    Ok(())
}

async fn run_inner_fallible(path: &str, opt: OptLevel, initial_gc_threshold: Option<usize>) -> anyhow::Result<()> {
    let path = Path::new(path);
    if !path.is_dir() {
        // TODO: make it also work with paths to files. need to adjust the execute_node_module call too,
        // since that needs a dir path
        bail!("Node project path currently needs to be a directory");
    }

    let package = std::fs::read_to_string(path.join("package.json")).context("Failed to read package.json")?;
    let package = serde_json::from_str::<Package>(&package)?;

    let entry = std::fs::read_to_string(path.join(package.main)).context("Failed to read entry point")?;

    let mut rt = Runtime::new(initial_gc_threshold).await;
    let scope = &mut rt.vm_mut().scope();

    execute_node_module(
        scope,
        path,
        path,
        &entry,
        opt,
        Rc::new(RefCell::new(FxHashMap::default())),
    )
    .map_err(|err| match err {
        (EvalError::Middle(errs), _) => anyhow!("{}", errs.formattable(&entry, true)),
        (EvalError::Exception(err), _) => anyhow!("{}", format_value(err.root(scope), scope).unwrap()),
    })?;

    Ok(())
}

/// Returns the `module` object
fn execute_node_module(
    scope: &mut LocalScope,
    dir_path: &Path,
    file_path: &Path,
    source: &str,
    opt: OptLevel,
    ongoing_requires: Rc<RefCell<FxHashMap<PathBuf, Value>>>,
) -> Result<Value, (EvalError, StringInterner)> {
    let exports = Value::Object(scope.register(NamedObject::new(scope)));
    let module = Value::Object(scope.register(NamedObject::new(scope)));
    let require = Value::Object(scope.register(RequireFunction {
        dir: dir_path.to_owned(),
        ongoing_requires: ongoing_requires.clone(),
        object: NamedObject::new(scope),
    }));
    module
        .set_property(scope, "exports".into(), PropertyValue::static_default(exports.clone()))
        .unwrap();
    scope
        .global()
        .set_property(scope, "module".into(), PropertyValue::static_default(module.clone()))
        .unwrap();
    scope
        .global()
        .set_property(scope, "exports".into(), PropertyValue::static_default(exports.clone()))
        .unwrap();
    scope
        .global()
        .set_property(scope, "require".into(), PropertyValue::static_default(require))
        .unwrap();

    ongoing_requires
        .borrow_mut()
        .insert(file_path.to_owned(), module.clone());

    scope.eval(source, opt)?;

    Ok(module)
}

#[derive(Debug, Trace)]
struct RequireFunction {
    dir: PathBuf,
    ongoing_requires: Rc<RefCell<FxHashMap<PathBuf, Value>>>,
    object: NamedObject,
}

impl Object for RequireFunction {
    delegate!(
        object,
        get_own_property_descriptor,
        set_property,
        delete_property,
        set_prototype,
        get_prototype,
        as_any,
        own_keys
    );

    fn type_of(&self) -> dash_vm::value::Typeof {
        dash_vm::value::Typeof::Function
    }

    fn apply(
        &self,
        scope: &mut LocalScope,
        _callee: dash_vm::gc::handle::Handle<dyn Object>,
        _this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        let Some(Value::String(arg)) = args.first() else {
            throw!(scope, Error, "require() expects a string argument");
        };

        let is_path = matches!(arg.chars().next(), Some('.' | '/' | '~'));
        if is_path {
            let canonicalized_path = match self.dir.join(&**arg).canonicalize() {
                Ok(v) => v,
                Err(err) => throw!(scope, Error, err.to_string()),
            };

            if let Some(module) = self.ongoing_requires.borrow().get(&canonicalized_path) {
                return Ok(module.clone());
            }

            let source = match std::fs::read_to_string(&canonicalized_path) {
                Ok(v) => v,
                Err(err) => throw!(scope, Error, err.to_string()),
            };

            let Some(parent_dir) = canonicalized_path.parent() else {
                throw!(
                    scope,
                    Error,
                    "Failed to get parent dir of {}",
                    canonicalized_path.display()
                );
            };

            let module = match execute_node_module(
                scope,
                parent_dir,
                &canonicalized_path,
                &source,
                OptLevel::default(),
                self.ongoing_requires.clone(),
            ) {
                Ok(v) => v,
                Err((EvalError::Exception(value), _)) => return Err(value.root(scope)),
                Err((EvalError::Middle(errs), _)) => {
                    throw!(scope, SyntaxError, "{}", errs.formattable(&source, true))
                }
            };

            module.get_property(scope, "exports".into())
        } else {
            // Resolve dependency
            todo!("Resolve external dependency");
        }
    }
}
