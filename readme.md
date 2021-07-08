# dash
ECMA-262 implementation in Rust.
This includes a source code lexer, parser, bytecode compiler and VM. 

[Try it in your browser](http://dash.y21_.repl.co/)

## ⚠️ WIP
This is a *WIP* and **not** yet production ready.

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
for (let i = 0; i < 100; ++i) {
    if (!(i % 15)) console.log("Fizzbuzz");
    else if (!(i % 5)) console.log("Fizz");
    else if (!(i % 3)) console.log("Buzz");
    else console.log(i);
}
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
- Cargo.toml
```toml
[dependencies]
dash = { git = "https://github.com/y21/dash", path = "core" }
```
- main.rs
```rs
fn main() {
    let source = "const x = 42; x * x";
    let res: f64 = dash::eval::<()>(source, None)
        .unwrap()     // → Option<Rc<RefCell<Value>>>
        .unwrap()     // → RefCell<Value>
        .borrow()     // → Ref<Value>
        .as_number(); // → f64

    // Result: 1764.0
    println!("Result: {:?}", res);
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