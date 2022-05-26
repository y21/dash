# dash
![Tests](https://github.com/y21/dash/actions/workflows/test.yml/badge.svg)
![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/y21/dash)

ECMA-262 implementation in pure Rust. 

## ⚠️ WIP
This is a *WIP* and **not** yet production ready. It is actively being worked on and the API is constantly changing.

## Goals
- Target ECMAScript 2015
- Heap/Bytecode snapshot support
- Compatibility
- Easily embeddable into any Rust application
- WebAssembly support
- Optional JIT

## Usage
### Using the CLI
```js
import * as http from '@std/http';

function* counter() {
    let num = 0;
    while (true) yield num++;
}

const numbers = counter();
const port = 3030;

http.listen(port, (ctx) => {
    const next = numbers.next();
    ctx.respond('Request count: ' + next.value);
});

console.log('Listening on port: ' + port);
```
```sh
# Install Rust
$ curl -sSf https://sh.rustup.rs | sh
# Clone repo
$ git clone https://github.com/y21/dash
# Build cli
$ cd dash/crates/dash-cli && cargo install --path .
# Optional: rename binary to `dashjs`
$ mv ~/.cargo/bin/dash-cli ~/.cargo/bin/dashjs
# Run the program (run with --help for help)
$ dashjs run example.js
```
Now open up your browser, navigate to http://localhost:3030, refresh a bunch of times and see the numbers go up.

### Embedding into a Rust application
Note that the API is not stable. Things are constantly changing, so your code may break at any time when bumping the version, which is why it is highly recommended to lock in to a specific revision for now.

- Cargo.toml
```toml
[dependencies]
dash-core = { git = "https://github.com/y21/dash" }
```
- main.rs
```rs
use dash_core as dash;
use dash::vm::Vm;

fn main() {
    let source = "const x = 42; x * x";

    let mut vm = Vm::new(Default::default());
    let result = vm.eval(source, Default::default()).expect("JS Exception");

    println!("Result: {}", match result {
        Value::Number(n) => n,
        _ => unreachable!()
    });
}
```
<sub>See `dash-cli/` for a more detailed example</sub>
