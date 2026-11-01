use dash_optimizer::OptLevel;
use dash_vm::params::VmParams;

pub fn params_from_args(args: &clap::ArgMatches) -> anyhow::Result<VmParams> {
    let initial_gc_threshold = args
        .get_one::<&str>("initial-gc-threshold")
        .map(|v| v.parse())
        .transpose()?;

    let experimental_objdump_jit = *args.get_one::<bool>("xobjdump-jit").unwrap();

    let opt = *args.get_one::<OptLevel>("opt").unwrap();

    Ok(VmParams {
        initial_gc_rss_threshold: initial_gc_threshold,
        experimental_objdump_jit,
        opt,
        ..Default::default()
    })
}
