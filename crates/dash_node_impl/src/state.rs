use std::cell::OnceCell;

use dash_middle::define_symbol_set;
use dash_proc_macro::Trace;
use dash_rt::typemap::Key;
use dash_vm::value::Value;
use dash_vm::Vm;

define_symbol_set!(#[derive(Trace)] NodeSymbols => [fs, fetch, path, parse, dir, events, EventEmitter, on, emit, stream, http, https, url, zlib, punycode]);

#[derive(Trace)]
pub struct State {
    pub sym: NodeSymbols,
    pub fs_cache: OnceCell<Value>,
    pub fetch_cache: OnceCell<Value>,
    pub path_cache: OnceCell<Value>,
    pub events_cache: OnceCell<Value>,
    pub stream_cache: OnceCell<Value>,
    pub http_cache: OnceCell<Value>,
    pub https_cache: OnceCell<Value>,
    pub url_cache: OnceCell<Value>,
    pub zlib_cache: OnceCell<Value>,
    pub punycode_cache: OnceCell<Value>,
}

impl State {
    pub fn new(vm: &mut Vm) -> Self {
        Self {
            sym: NodeSymbols::new(&mut vm.interner),
            fs_cache: OnceCell::new(),
            fetch_cache: OnceCell::new(),
            path_cache: OnceCell::new(),
            stream_cache: OnceCell::new(),
            events_cache: OnceCell::new(),
            http_cache: OnceCell::new(),
            https_cache: OnceCell::new(),
            url_cache: OnceCell::new(),
            zlib_cache: OnceCell::new(),
            punycode_cache: OnceCell::new(),
        }
    }
}

pub struct Nodejs;
impl Key for Nodejs {
    type State = State;
}

pub fn state_mut(vm: &mut Vm) -> &mut State {
    &mut dash_rt::state::State::from_vm_mut(vm).store[Nodejs]
}
