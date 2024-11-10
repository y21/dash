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
    inspect,
    Stream,
    Readable,
    Inflate,
    querystring,
    timers,
    Buffer,
    buffer,
    alloc,
    writeUInt32BE,
    writeUInt32LE
]);
