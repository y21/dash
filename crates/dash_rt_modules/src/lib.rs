use dash_rt::module::ModuleLoader;

pub fn init_modules() -> Box<dyn ModuleLoader> {
    let module = dash_rt_http::HttpModule
        .or(dash_rt_fs::FsModule)
        .or(dash_rt_fetch::FetchModule)
        .or(dash_rt_timers::TimersModule::new())
        .or(dash_dlloader::DllModule::new())
        .or(dash_rt_script_modules::ScriptModule::new());

    Box::new(module)
}
