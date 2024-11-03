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
use dash_vm::gc::ObjectId;
use dash_vm::localscope::LocalScope;
use dash_vm::value::array::Array;
use dash_vm::value::object::{NamedObject, Object, PropertyValue};
use dash_vm::value::{Root, Unpack, Unrooted, Value, ValueKind};
use dash_vm::{delegate, throw, Vm};
use package::Package;
use rustc_hash::FxHashMap;
use state::Nodejs;

mod assert;
mod events;
mod native;
mod package;
mod path;
mod state;
mod stream;
mod symbols;
mod util;

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
            base_dir: match path.parent() {
                Some(p) => p.to_path_buf(),
                None => env::current_dir()?,
            },
            metadata: Package::default_with_entry(path.into()),
        }
    };

    let entry_path = if path.is_dir() {
        path.join(&package_state.metadata.main)
    } else {
        package_state.metadata.main.clone()
    };

    let entry = std::fs::read_to_string(&entry_path)?;

    let global_state = Rc::new(GlobalState {
        node_modules_dir: package_state.base_dir.join("node_modules"),
        ongoing_requires: RefCell::new(FxHashMap::default()),
    });

    let mut rt = Runtime::new(initial_gc_threshold).await;
    let state = state::State::new(rt.vm_mut());
    State::from_vm_mut(rt.vm_mut()).store.insert(Nodejs, state);

    rt.vm_mut().with_scope(|scope| {
        let global = scope.global();
        let global_k = scope.intern("global");
        let process_k = scope.intern("process");
        global
            .clone()
            .set_property(
                scope,
                global_k.into(),
                PropertyValue::static_default(Value::object(global)),
            )
            .unwrap();
        let process = create_process_object(scope);
        global
            .set_property(scope, process_k.into(), PropertyValue::static_default(process.into()))
            .unwrap();

        anyhow::Ok(
            execute_node_module(
                scope,
                entry_path.parent().unwrap(),
                &entry_path,
                &entry,
                opt,
                global_state,
                Rc::new(package_state),
            )
            .map_err(|err| match err {
                (EvalError::Middle(errs), entry) => anyhow!("{}", errs.formattable(&entry, true)),
                (EvalError::Exception(err), ..) => anyhow!("{}", format_value(err.root(scope), scope).unwrap()),
            })?,
        )
    })?;

    if rt.state().needs_event_loop() {
        rt.run_event_loop().await;
    }

    Ok(())
}

fn create_process_object(sc: &mut LocalScope<'_>) -> ObjectId {
    let obj = NamedObject::new(sc);
    let env = NamedObject::new(sc);
    let env = sc.register(env);
    let env_k = sc.intern("env");
    obj.set_property(sc, env_k.into(), PropertyValue::static_default(env.into()))
        .unwrap();

    let argv_k = sc.intern("argv");
    let argv = env::args()
        .map(|arg| PropertyValue::static_default(Value::string(sc.intern(arg).into())))
        .collect::<Vec<_>>();
    let argv = Array::from_vec(sc, argv);
    let argv = sc.register(argv);
    obj.set_property(sc, argv_k.into(), PropertyValue::static_default(argv.into()))
        .unwrap();

    let versions_k = sc.intern("versions");
    let dash_k = sc.intern("dash");
    let versions = NamedObject::new(sc);
    let version = sc.intern(env!("CARGO_PKG_VERSION"));
    versions
        .set_property(
            sc,
            dash_k.into(),
            PropertyValue::static_default(Value::string(version.into())),
        )
        .unwrap();
    let versions = sc.register(versions);
    obj.set_property(sc, versions_k.into(), PropertyValue::static_default(versions.into()))
        .unwrap();

    sc.register(obj)
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
    let exports = Value::object(scope.register(NamedObject::new(scope)));
    let module = Value::object(scope.register(NamedObject::new(scope)));
    let require = Value::object(scope.register(RequireFunction {
        current_dir: dir_path.to_owned(),
        state: global_state.clone(),
        package,
        object: NamedObject::new(scope),
    }));
    let key = scope.intern("exports");
    module
        .set_property(scope, key.into(), PropertyValue::static_default(exports))
        .unwrap();

    global_state
        .ongoing_requires
        .borrow_mut()
        .insert(file_path.to_owned(), module);

    let mut code = String::from("(function(exports, module, require) {\n");
    code += source;
    code += "\n})";

    let fun = match scope.eval(&code, opt) {
        Ok(v) => v.root(scope),
        Err(err) => return Err((err, code)),
    };

    fun.apply(scope, Value::undefined(), vec![exports, module, require])
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

    fn type_of(&self, _: &Vm) -> dash_vm::value::Typeof {
        dash_vm::value::Typeof::Function
    }

    fn apply(
        &self,
        scope: &mut LocalScope,
        _callee: dash_vm::gc::ObjectId,
        _this: Value,
        args: Vec<Value>,
    ) -> Result<Unrooted, Unrooted> {
        let Some(ValueKind::String(raw_arg)) = args.first().unpack() else {
            throw!(scope, Error, "require() expects a string argument");
        };
        let exports = scope.intern("exports");
        let mut arg = raw_arg.res(scope).to_owned();

        let is_path = matches!(arg.chars().next(), Some('.' | '/' | '~'));
        let result = if is_path {
            if !arg.ends_with(".js") && !arg.ends_with(".json") {
                if std::fs::metadata(self.current_dir.join(&arg)).is_ok_and(|md| md.is_dir()) {
                    arg += "/index.js";
                } else {
                    arg += ".js";
                }
            }

            let canonicalized_path = match self.current_dir.join(&arg).canonicalize() {
                Ok(v) => v,
                Err(err) => throw!(scope, Error, err.to_string()),
            };
            debug!("require path module {}", canonicalized_path.display());

            if let Some(module) = self.state.ongoing_requires.borrow().get(&canonicalized_path) {
                debug!(%arg, "resolved module (cache)");
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
        } else if let Some(o) = native::load_native_module(scope, raw_arg)? {
            Ok(o.into())
        } else {
            // Resolve dependency in node_modules
            // If we have something like `require('a/b/c')`,
            // try looking for modules (in the following order):
            // - node_modules/a/package.json
            // - node_modules/a/b/package.json
            // - node_modules/a/b/c/package.json

            let components = Path::new(&arg).components().collect::<Vec<_>>();

            let module = (0..components.len())
                .map(|c| self.state.node_modules_dir.join(PathBuf::from_iter(&components[0..=c])))
                .find_map(|v| process_package_json(&v).ok().map(|pkg| (pkg, v)));

            let (package_state, dir_path) = match module {
                Some((package_state, dir_path)) => (package_state, dir_path),
                None => throw!(scope, Error, "Failed to load module {}", arg),
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
