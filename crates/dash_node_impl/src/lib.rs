use std::cell::RefCell;
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::{Context, anyhow};
use dash_log::debug;
use dash_middle::interner::sym;
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
use dash_vm::value::function::args::CallArgs;
use dash_vm::value::function::native::register_native_fn;
use dash_vm::value::object::{Object, OrdObject, PropertyValue, This};
use dash_vm::value::propertykey::ToPropertyKey;
use dash_vm::value::{Root, Unpack, Unrooted, Value, ValueKind};
use dash_vm::{Vm, delegate, extract, throw};
use package::Package;
use rustc_hash::FxHashMap;
use state::Nodejs;
use symbols::NodeSymbols;

mod assert;
mod buffer;
mod child_process;
mod events;
mod native;
mod os;
mod package;
mod path;
mod state;
mod stream;
mod symbols;
mod time_ext;
mod util;
mod zlib;

pub struct NodeRunArgs<'a, S> {
    pub path: &'a str,
    pub opt: OptLevel,
    pub initial_gc_threshold: Option<usize>,
    pub script_args: S,
}

pub fn run_with_nodejs_mnemnoics<'a>(args: NodeRunArgs<'a, impl Iterator<Item = &'a str>>) -> anyhow::Result<()> {
    let tokio_rt = tokio::runtime::Runtime::new()?;

    tokio_rt.block_on(async move {
        if let Err(err) = run_inner_fallible(args).await {
            eprintln!("{err}");
        }
    });

    Ok(())
}

async fn run_inner_fallible<'a>(
    NodeRunArgs {
        path,
        opt,
        initial_gc_threshold,
        script_args,
    }: NodeRunArgs<'a, impl Iterator<Item = &'a str>>,
) -> anyhow::Result<()> {
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
        path.join(&package_state.metadata.main).canonicalize()?
    } else {
        package_state.metadata.main.clone().canonicalize()?
    };

    let entry = std::fs::read_to_string(&entry_path)?;

    let global_state = Rc::new(GlobalState {
        node_modules_dir: package_state.base_dir.join("node_modules"),
        ongoing_requires: RefCell::new(FxHashMap::default()),
    });

    let mut rt = Runtime::new(initial_gc_threshold);
    let state @ state::State {
        sym:
            NodeSymbols {
                global: global_sym,
                process: process_sym,
                Buffer: buffer_sym,
                setTimeout: set_timeout_sym,
                time: time_sym,
                timeEnd: time_end_sym,
                ..
            },
        ..
    } = state::State::new(rt.vm_mut());
    State::from_vm_mut(rt.vm_mut()).store.insert(Nodejs, state);

    rt.vm_mut().with_scope(|scope| {
        let global = scope.global();
        global
            .clone()
            .set_property(
                global_sym.to_key(scope),
                PropertyValue::static_default(Value::object(global)),
                scope,
            )
            .unwrap();

        let process = create_process_object(scope, script_args)?;
        global
            .set_property(
                process_sym.to_key(scope),
                PropertyValue::static_default(process.into()),
                scope,
            )
            .unwrap();

        let buffer = buffer::init_module(scope).unwrap();
        global
            .set_property(buffer_sym.to_key(scope), PropertyValue::static_default(buffer), scope)
            .unwrap();
        let timer = dash_rt_timers::import(scope).unwrap();
        let set_timeout = timer
            .get_property(set_timeout_sym.to_key(scope), scope)
            .unwrap()
            .root(scope);
        global
            .set_property(
                set_timeout_sym.to_key(scope),
                PropertyValue::static_default(set_timeout),
                scope,
            )
            .unwrap();

        let console = global
            .get_property(sym::console.to_key(scope), scope)
            .unwrap()
            .root(scope);
        console
            .set_property(
                time_sym.to_key(scope),
                PropertyValue::static_default(register_native_fn(scope, time_sym, time_ext::console_time).into()),
                scope,
            )
            .unwrap();
        console
            .set_property(
                time_end_sym.to_key(scope),
                PropertyValue::static_default(
                    register_native_fn(scope, time_end_sym, time_ext::console_time_end).into(),
                ),
                scope,
            )
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

    rt.run_event_loop().await;

    Ok(())
}

fn create_process_object<'a>(
    sc: &mut LocalScope<'_>,
    script_args: impl Iterator<Item = &'a str>,
) -> anyhow::Result<ObjectId> {
    let obj = OrdObject::new(sc);
    let env = OrdObject::new(sc);
    let env = sc.register(env);
    let env_k = sc.intern("env");
    obj.set_property(env_k.to_key(sc), PropertyValue::static_default(env.into()), sc)
        .unwrap();

    let current_exe = env::current_exe().context("failed to get executable path")?;
    let current_exe = current_exe.to_str().context("invalid utf-8 in executable path")?;

    let argv_k = sc.intern("argv");
    let mut argv = Vec::new();
    argv.push(PropertyValue::static_default(Value::string(
        sc.intern(current_exe).into(),
    )));

    for arg in script_args {
        argv.push(PropertyValue::static_default(Value::string(sc.intern(arg).into())));
    }

    let argv = Array::from_vec(argv, sc);
    let argv = sc.register(argv);
    obj.set_property(argv_k.to_key(sc), PropertyValue::static_default(argv.into()), sc)
        .unwrap();

    let versions_k = sc.intern("versions");
    let dash_k = sc.intern("dash");
    let versions = OrdObject::new(sc);
    let version = sc.intern(env!("CARGO_PKG_VERSION"));
    versions
        .set_property(
            dash_k.to_key(sc),
            PropertyValue::static_default(Value::string(version.into())),
            sc,
        )
        .unwrap();
    let versions = sc.register(versions);
    obj.set_property(
        versions_k.to_key(sc),
        PropertyValue::static_default(versions.into()),
        sc,
    )
    .unwrap();

    Ok(sc.register(obj))
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
    let dir_path = dir_path.canonicalize().unwrap();
    let file_path = file_path.canonicalize().unwrap();

    debug!(?dir_path, ?file_path);
    let exports = Value::object(scope.register(OrdObject::new(scope)));
    let module = Value::object(scope.register(OrdObject::new(scope)));
    let require = Value::object(scope.register(RequireFunction {
        current_dir: dir_path.to_owned(),
        state: global_state.clone(),
        package,
        object: OrdObject::new(scope),
    }));
    let key = scope.intern("exports");
    module
        .set_property(key.to_key(scope), PropertyValue::static_default(exports), scope)
        .unwrap();

    global_state
        .ongoing_requires
        .borrow_mut()
        .insert(file_path.to_owned(), module);

    let mut code = String::from("(function(exports, module, require, __dirname, __filename) {\n");
    code += source;
    code += "\n})";

    let fun = match scope.eval(&code, opt) {
        Ok(v) => v.root(scope),
        Err(err) => return Err((err, code)),
    };

    let dirname = Value::string(scope.intern(dir_path.to_str().expect("invalid utf-8 path")).into());
    let filename = Value::string(scope.intern(file_path.to_str().expect("invalid utf-8 path")).into());
    fun.apply(
        This::default(),
        [exports, module, require, dirname, filename].into(),
        scope,
    )
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
    object: OrdObject,
}

impl Object for RequireFunction {
    delegate!(
        object,
        get_own_property_descriptor,
        set_property,
        delete_property,
        set_prototype,
        get_prototype,
        own_keys
    );

    fn type_of(&self, _: &Vm) -> dash_vm::value::Typeof {
        dash_vm::value::Typeof::Function
    }

    fn apply(
        &self,
        _callee: dash_vm::gc::ObjectId,
        _this: This,
        args: CallArgs,
        scope: &mut LocalScope,
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
                return module.get_property(exports.to_key(scope), scope);
            }

            let source = match std::fs::read_to_string(&canonicalized_path) {
                Ok(v) => v,
                Err(err) => throw!(scope, Error, err.to_string()),
            };

            if canonicalized_path.extension() == Some(OsStr::new("json")) {
                match dash_vm::json::parser::Parser::new(source.as_bytes(), scope).parse() {
                    Ok(val) => Ok(val.into()),
                    Err(err) => throw!(scope, SyntaxError, "{}", err.to_string()),
                }
            } else {
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

                module.get_property(exports.to_key(scope), scope)
            }
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

            let mut file_path = dir_path.join(&package_state.metadata.main);
            if file_path.extension().is_none() {
                file_path.set_extension("js");
            }
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

            module.get_property(exports.to_key(scope), scope)
        };
        debug!(%arg, "resolved module");
        result
    }

    extract!(self);
}
