# dash
![Tests](https://github.com/y21/dash/actions/workflows/test.yml/badge.svg)
![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/y21/dash)

ECMA-262 implementation in pure Rust. 

[Try it in your browser](http://dash.y21_.repl.co/)


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

for (const number of counter()) console.log(number);
```
```sh
# Install Rust
$ curl -sSf https://sh.rustup.rs | sh
# Clone repo
$ git clone https://github.com/y21/dash
# Build cli
$ cd dash/cli && cargo install --path .
# Rename `cli` to `dashjs`
$ mv ~/.cargo/bin/cli ~/.cargo/bin/dashjs
# Run the program (run with --help for help)
$ dashjs example.js
```
### Embedding into a JavaScript application
This implementation can be used in another JavaScript engine that supports WebAssembly. To do so, build `wasm/` and include `dash.ts` in your application. In the future, this project will be published on npm to make this easier. There will be an example for this soon.

### Embedding into a Rust application
Note that the API is not stable. Things are constantly changing, so your code may break at any time when bumping the version, which is why it is highly recommended to lock in to a specific revision for now.

- Cargo.toml
```toml
[dependencies]
dash = { git = "https://github.com/y21/dash", path = "core" }
```
- main.rs
```rs
fn main() {
    let source = "const x = 42; x * x";
    let (result, vm) = dash::eval::<()>(source, None)
        .unwrap(); // unwrap, we know the source string is valid!
    
    let result: f64 = result
        .unwrap() // unwrap, we know there *is* a value
        .borrow(&vm)
        .as_number();

    // Result: 1764.0
    println!("Result: {:?}", result);
}
```
<sub>See `cli/` for a more detailed example</sub>

## Progress
A list of supported ECMAScript features can be found in progress.md.

## Project structure
- `cli/`: A command line program that embeds the core engine and runtime, used to run JavaScript code. End users will use this. 
- `core/`: A JavaScript engine (lexer, parser, compiler, VM) that can be embedded into any application to run JavaScript code.
- `runtime/`: A runtime that adds additional features that are often used by JavaScript applications, such as access to the file system.
- `testrunner/` and `test262/`: ECMAScript spec compliance testing. Not used yet because of lack of features required for running tests.
- `wasm/`: WebAssembly back- and frontend. Provides bindings to core project and makes it possible to embed the engine in the browser.