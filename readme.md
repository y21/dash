# dash
ECMA-262 implementation in Rust.
This includes a source code lexer, parser, bytecode compiler and VM. 

## ⚠️ WIP
This is a *WIP* and **not** yet production ready.

## Goals
- Target ECMAScript 2015
- Heap/Bytecode snapshot support
- Compatibility
- Easily embeddable into any Rust application
- WebAssembly support

## Progress
A list of supported ECMAScript features can be found in progress.md.

## Project structure
- `cli/`: A command line program that embeds the core engine and runtime, used to run JavaScript code. End users will use this. 
- `core/`: A JavaScript engine (lexer, parser, compiler, VM) that can be embedded into any application to run JavaScript code.
- `runtime/`: A runtime that adds additional features that are often used by JavaScript applications, such as access to the file system.
- `testrunner/` and `test262/`: ECMAScript spec compliance testing. Not used yet because of lack of features required for running tests.
- `wasm/`: WebAssembly back- and frontend. Provides bindings to core project and makes it possible to embed the engine in the browser.
- `site/`: A site that embeds the engine and a PoC of the project. It makes testing features easy as it doesn't require building the source.