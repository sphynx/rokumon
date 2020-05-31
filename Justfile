build:
    cargo build --release

wasm: build
    wasm-pack build rokumon_wasm

serve: wasm
    cd www && npm install && npm run serve

test:
    cargo test

play:
    cargo run -p rokumon_console_ui
