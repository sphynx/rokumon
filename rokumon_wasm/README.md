# Info

Contains WASM-friendly interface and JS bindings generated with wasm-bindgen.

# How to build

1. Install `wasm-pack`:

```
cargo install wasm-pack
```

or follow instruction on [wasm-pack page](https://rustwasm.github.io/wasm-pack/installer/)

2. Run it:

```
wasm-pack build
```

It should compile the code for `wasm32-unknown-unknown` target, build
and install `wasm-bindgen`, generate JS binding and prepare a fully
function JS package which can be imported from the webpages.

Expected output might look like this:

```
➜  rokumon_wasm git:(workspace) ✗ wasm-pack build
[INFO]: 🎯  Checking for the Wasm target...
[INFO]: 🌀  Compiling to Wasm...
    Finished release [optimized + debuginfo] target(s) in 0.04s
[INFO]: ⬇️  Installing wasm-bindgen...
[INFO]: Optimizing wasm binaries with `wasm-opt`...
[INFO]: Optional fields missing from Cargo.toml: 'description', 'repository', and 'license'. These are not necessary, but recommended
[INFO]: ✨   Done in 0.37s
[INFO]: 📦   Your wasm pkg is ready to publish at /Users/sphynx/Code/rust/rokumon/rokumon_wasm/pkg.
```
