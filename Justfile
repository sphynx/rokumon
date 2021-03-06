build:
    cargo build --release

wasm:
    wasm-pack build --release rokumon_wasm
    @ # `wasm-objdump` is part of WASM binary toolkit: https://github.com/WebAssembly/wabt
    wasm-objdump rokumon_wasm/pkg/*.wasm -x -j export

build_prod: wasm
    cd www && npm run build

serve: wasm
    cd www && npm install && npm start

test:
    cargo test

play:
    cargo run -p rokumon_console_ui

clean:
    cargo clean

deploy: build_prod
    rsync --checksum --progress -ave ssh www/build/* sphynx@iveselov.info:/srv/projects/rokumon
