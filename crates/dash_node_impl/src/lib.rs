use std::cell::RefCell;
use std::env;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::{anyhow, Context};
use dash_log::debug;
use dash_middle::parser::error::IntoFormattableErrors;
use dash_optimizer::OptLevel;
use dash_proc_macro::Trace;
use dash_rt::format_value;
use dash_rt::runtime::Runtime;
use dash_rt::state::State;
use dash_vm::eval::EvalError;
use dash_vm::localscope::LocalScope;
use dash_vm::value::object::{NamedObject, Object, PropertyValue};
use dash_vm::value::{Root, Unrooted, Value};
use dash_vm::{delegate, throw};
use package::Package;
use rustc_hash::FxHashMap;
use state::Nodejs;

mod events;
mod native;
mod package;
mod path;
mod state;

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
    let package_state = if path.is_dir() {
        process_package_json(path)?
    } else {
        PackageState {
            base_dir: path.parent().unwrap_or(&env::current_dir()?).into(),
            metadata: Package::default_with_entry(path.into()),
        }
    };

    let entry = if path.is_dir() {
        std::fs::read_to_string(path.join(&package_state.metadata.main))
    } else {
        std::fs::read_to_string(&package_state.metadata.main)
    }?;

    let global_state = Rc::new(GlobalState {
        node_modules_dir: package_state.base_dir.join("node_modules"),
        ongoing_requires: RefCell::new(FxHashMap::default()),
    });

    let mut rt = Runtime::new(initial_gc_threshold).await;
    let state = state::State::new(rt.vm_mut());
    State::from_vm_mut(rt.vm_mut()).store.insert(Nodejs, state);

    rt.vm_mut().with_scope(|scope| {
        anyhow::Ok(
            execute_node_module(scope, path, path, &entry, opt, global_state, Rc::new(package_state)).map_err(
                |err| match err {
                    (EvalError::Middle(errs), entry) => anyhow!("{}", errs.formattable(&entry, true)),
                    (EvalError::Exception(err), ..) => anyhow!("{}", format_value(err.root(scope), scope).unwrap()),
                },
            )?,
        )
    })?;

    if rt.state().needs_event_loop() {
        rt.run_event_loop().await;
    }

    Ok(())
}

fn process_package_json(path: &Path) -> Result<PackageState, anyhow::Error> {
    let package = std::fs::read_to_string(path.join("package.json")).context("Failed to read package.json")?;
    let package = serde_json::from_str::<Package>(&package).context("Failed to parse package.json")?;
    let base_dir = path.to_owned();
    Ok(PackageState {
        metadata: package,
        base_dir,
    })
}

/// Returns the `module` object
fn execute_node_module(
    scope: &mut LocalScope,
    dir_path: &Path,
    file_path: &Path,
    source: &str,
    opt: OptLevel,
    global_state: Rc<GlobalState>,
    package: Rc<PackageState>,
) -> Result<Value, (EvalError, String)> {
    debug!(?dir_path, ?file_path);
    let exports = Value::Object(scope.register(NamedObject::new(scope)));
    let module = Value::Object(scope.register(NamedObject::new(scope)));
    let require = Value::Object(scope.register(RequireFunction {
        current_dir: dir_path.to_owned(),
        state: global_state.clone(),
        package,
        object: NamedObject::new(scope),
    }));
    let key = scope.intern("exports");
    module
        .set_property(scope, key.into(), PropertyValue::static_default(exports.clone()))
        .unwrap();

    global_state
        .ongoing_requires
        .borrow_mut()
        .insert(file_path.to_owned(), module.clone());

    let mut code = String::from("(function(exports, module, require) {\n");
    code += source;
    code += "\n})";

    let fun = match scope.eval(&code, opt) {
        Ok(v) => v.root(scope),
        Err(err) => return Err((err, code)),
    };

    fun.apply(scope, Value::undefined(), vec![exports, module.clone(), require])
        .map_err(|err| (EvalError::Exception(err), code))?;

    Ok(module)
}

#[derive(Debug, Trace)]
struct PackageState {
    metadata: Package,
    /// Path to the base directory of the package
    base_dir: PathBuf,
}

#[derive(Debug, Trace)]
struct GlobalState {
    node_modules_dir: PathBuf,
    ongoing_requires: RefCell<FxHashMap<PathBuf, Value>>,
}

#[derive(Debug, Trace)]
struct RequireFunction {
    /// Path to the current directory
    current_dir: PathBuf,
    package: Rc<PackageState>,
    state: Rc<GlobalState>,
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
        _callee: dash_vm::gc::handle::Handle,
        _this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        let Some(Value::String(arg)) = args.first() else {
            throw!(scope, Error, "require() expects a string argument");
        };
        let exports = scope.intern("exports");
        let raw_arg = arg;
        let mut arg = arg.res(scope).to_owned();
        debug!(%arg, "require node module");

        let is_path = matches!(arg.chars().next(), Some('.' | '/' | '~'));
        let result = if is_path {
            if !arg.ends_with(".js") && !arg.ends_with(".json") {
                arg += ".js";
            }

            let canonicalized_path = match self.current_dir.join(&arg).canonicalize() {
                Ok(v) => v,
                Err(err) => throw!(scope, Error, err.to_string()),
            };

            if let Some(module) = self.state.ongoing_requires.borrow().get(&canonicalized_path) {
                return module.get_property(scope, exports.into());
            }

            let source = match std::fs::read_to_string(&canonicalized_path) {
                Ok(v) => v,
                Err(err) => throw!(scope, Error, err.to_string()),
            };

            let module = match execute_node_module(
                scope,
                canonicalized_path.parent().unwrap(),
                &canonicalized_path,
                &source,
                OptLevel::default(),
                self.state.clone(),
                self.package.clone(),
            ) {
                Ok(v) => v,
                Err((EvalError::Exception(value), ..)) => return Err(value),
                Err((EvalError::Middle(errs), source)) => {
                    throw!(scope, SyntaxError, "{}", errs.formattable(&source, true))
                }
            };

            module.get_property(scope, exports.into())
        } else if let Some(o) = native::load_native_module(scope, *raw_arg)? {
            Ok(o.into())
        } else {
            // Resolve dependency in node_modules
            let dir_path = self.state.node_modules_dir.join(&arg);

            let package_state = match process_package_json(&dir_path) {
                Ok(p) => p,
                Err(e) => throw!(scope, Error, "Failed to load module {arg}: {}", e),
            };
            let file_path = dir_path.join(&package_state.metadata.main);
            let source = std::fs::read_to_string(&file_path).unwrap();

            let module = match execute_node_module(
                scope,
                file_path.parent().unwrap(),
                &file_path,
                &source,
                OptLevel::default(),
                self.state.clone(),
                Rc::new(package_state),
            ) {
                Ok(v) => v,
                Err((EvalError::Exception(value), ..)) => return Err(value),
                Err((EvalError::Middle(errs), source)) => {
                    throw!(scope, SyntaxError, "{}", errs.formattable(&source, true))
                }
            };

            module.get_property(scope, exports.into())
        };
        debug!(%arg, "resolved module");
        result
    }
}
