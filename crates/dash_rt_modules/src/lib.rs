use dash_rt::module::ModuleLoader;
use dash_rt::module::NoopModule;

pub fn init_modules() -> Box<dyn ModuleLoader> {
    let module = NoopModule;
    #[cfg(feature = "http")]
    let module = module.or(dash_rt_http::HttpModule);
    #[cfg(feature = "fs")]
    let module = module.or(dash_rt_fs::FsModule);
    #[cfg(feature = "fetch")]
    let module = module.or(dash_rt_fetch::FetchModule);
    #[cfg(feature = "modules")]
    let module = module.or(dash_rt_script_modules::ScriptModule::new());
    #[cfg(feature = "timers")]
    let module = module.or(dash_rt_timers::TimersModule);
    #[cfg(feature = "dll")]
    let module = module.or(dash_dlloader::DllModule);

    Box::new(module)
}
