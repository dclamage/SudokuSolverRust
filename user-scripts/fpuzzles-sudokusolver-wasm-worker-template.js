importScripts('${WASM_JS_URL}');

const { solve } = wasm_bindgen;

async function init_wasm_worker() {
    await wasm_bindgen("${WASM_BIN_URL}");
    self.onmessage = async event => {
        solve(event.data, response => self.postMessage(response));
    }
}

init_wasm_worker();