use dash_middle::define_symbol_set;
use dash_proc_macro::Trace;

define_symbol_set!(#[derive(Trace)] NodeSymbols => [
    assert,
    fs,
    fetch,
    path,
    parse,
    dir,
    events,
    util,
    EventEmitter,
    on,
    emit,
    stream,
    http,
    https,
    url,
    zlib,
    punycode,
    inherits,
    Stream,
    Readable,
    Inflate,
    querystring,
    timers
]);
