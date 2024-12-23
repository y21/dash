# dash

![Tests](https://github.com/y21/dash/actions/workflows/test.yml/badge.svg)
![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/y21/dash)

Experimental JavaScript implementation in Rust.

## ⚠️ WIP

This is a _WIP_ and **not** yet production ready. It is actively being worked on and the API is constantly changing.

Current status: Not recommended for use in real projects. Feel free to experiment. The majority of language constructs are implemented and "work fine". It currently passes around 25% of test262.

## Usage

### Using the CLI

```js
const numbers = [1, 2, 3, 4, 5];
const sum = numbers.reduce((acc, num) => acc + num, 0);

console.log(`Sum of array: ${sum}`);
```

## Install

```sh
# Install Rust
$ curl -sSf https://sh.rustup.rs | sh

# Build project
$ cargo install --git https://github.com/y21/dash dash-cli

# Optional: rename binary to `dashjs`
$ mv ~/.cargo/bin/dash-cli ~/.cargo/bin/dashjs

# Run the program (run with --help for help)
$ dashjs run example.js
```

### Embedding into a Rust application

Note that the API is very unstable. Things are constantly changing, so your code may break at any time when bumping the version, which is why it is highly recommended to lock in to a specific revision for now.
The MSRV for this project is the version that is currently stable. No nightly should be required to build the project or to use it as a library (except for the custom linter in `lints/` which is only useful for development).

- Cargo.toml

```toml
[dependencies]
dash_vm = { git = "https://github.com/y21/dash", features = ["eval"], rev = "9401b84" }
```

> The `eval` feature exposes a convenience `eval()` method on the `Vm` struct
> that lets you specify a JavaScript source string directly instead of having to pass the different IRs around.

- main.rs

```rs
use dash_vm::Vm;
use dash_vm::value::Root;
use dash_vm::value::ops::conversions::ValueConversion;

fn main() {
    let source = "const x = 42; x * x";
    let mut vm = Vm::new(Default::default());
    let mut scope = vm.scope();
    let result = scope
        .eval(source, Default::default())
        .unwrap()
        .root(&mut scope);

    println!("Result: {}", result.to_number(&mut scope).unwrap());
}
```

### Node compatibility

There's experimental support for running scripts that use NodeJS APIs. If you want to try it out, pass `--features nodejs` to the cargo install/build command.
When running dash, you can then pass `--node` and various node-specific things will be available to the JS environment, such as the `require` function.
