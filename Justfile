build:
    cargo build --release

wasm:
    wasm-pack build rokumon_wasm
    @ # `wasm-objdump` is part of WASM binary toolkit: https://github.com/WebAssembly/wabt
    wasm-objdump rokumon_wasm/pkg/*.wasm -x -j export

serve: wasm
    cd www && npm install && npm run serve

test:
    cargo test

play:
    cargo run -p rokumon_console_ui

clean:
    cargo clean
