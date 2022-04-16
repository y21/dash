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
// example.js
function* counter() {
    let num = 0;
    while (true) yield num++;
}

const numbers = counter();
let current;
while (!(current = numbers.next()).done) {
    console.log(current.value);
}

```
```sh
# Install Rust
$ curl -sSf https://sh.rustup.rs | sh
# Clone repo
$ git clone https://github.com/y21/dash
# Build cli
$ cd dash && cargo install --path dash-cli
# Optional: rename binary to `dashjs`
$ mv ~/.cargo/bin/dash-cli ~/.cargo/bin/dashjs
# Run the program (run with --help for help)
$ dashjs run example.js
```

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

fn main() {
    let source = "const x = 42; x * x";
    let (result, vm) = dash::eval(source, Default::default()).unwrap();

    println!("Result: {:?}", result);
}
```
<sub>See `dash-cli/` for a more detailed example</sub>

## Project structure
- `dash-cli/`: A command line program that embeds the core engine and runtime, used to run JavaScript code. End users will use this. 
- `dash-core/`: A JavaScript engine (lexer, parser, compiler, VM) that can be embedded into any application to run JavaScript code.
- `dash-rt/`: A runtime that adds additional features that are often used by JavaScript applications, such as access to the file system.
- `testrunner/` and `test262/`: ECMAScript spec compliance testing. Not used yet because of lack of features required for running tests.
- `dash-wasm/`: WebAssembly back- and frontend. Provides bindings to core project and makes it possible to embed the engine in the browser.
