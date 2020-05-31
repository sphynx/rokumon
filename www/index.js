import * as wasm from "rokumon_wasm";

const out = document.getElementById("out");
const game_descr = wasm.init_game(0);

out.innerHTML += game_descr;