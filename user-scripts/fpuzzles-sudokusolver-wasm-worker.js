importScripts('http://localhost:3000/sudoku_solver_wasm.js');

const { solve } = wasm_bindgen;

async function init_wasm_worker() {
    await wasm_bindgen("http://localhost:3000/sudoku_solver_wasm_bg.wasm");
    self.onmessage = async event => {
        solve(event.data, response => self.postMessage(response));
    }
}

init_wasm_worker();