mod utils;

use standard_constraints::message_handler::*;
use sudoku_solver_lib::prelude::Cancellation;
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct SendResultWasm {
    receive_result: js_sys::Function,
}

impl SendResultWasm {
    fn new(receive_result: &js_sys::Function) -> Self {
        Self { receive_result: receive_result.clone() }
    }
}

impl SendResult for SendResultWasm {
    fn send_result(&mut self, result: &str) {
        let this = JsValue::NULL;
        let args = js_sys::Array::of1(&JsValue::from_str(result));
        let _ = self.receive_result.call1(&this, &args);
    }
}

#[wasm_bindgen]
pub fn solve(message: &str, receive_result: &js_sys::Function) {
    set_panic_hook();

    let send_result = Box::new(SendResultWasm::new(receive_result));
    let mut message_handler = MessageHandler::new(send_result);
    message_handler.handle_message(message, Cancellation::default());
}
