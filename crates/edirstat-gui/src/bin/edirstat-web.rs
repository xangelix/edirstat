// -- Clippy Denies --
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
// --- Clippy Lint Groups & Specific Warnings ---
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::needless_return)]
// --- Allowed Lints (Overrides) ---
// The wasm entrypoint is allowed to abort loudly: there is no caller left to
// report to if the DOM or eframe fails to initialize.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::needless_return)]

//! eDirStat web frontend: an egui snapshot viewer running in the browser.
//! Open `.edst` / `.edst.zst` snapshot files produced by the native scanner
//! via the built-in file picker.

#[cfg(target_family = "wasm")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    console_error_panic_hook::set_once();
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    // SAFETY: called exactly once at program start, before any other code that
    // could rely on initialized statics. Required when linking with shared
    // memory (atomics target feature), where the runtime start is not invoked
    // automatically.
    unsafe { __wasm_call_ctors() };

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("edirstat_canvas")
            .expect("Failed to find #edirstat_canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("#edirstat_canvas was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|_cc| {
                    Ok(Box::new(edirstat_gui::GuiApp::new(
                        std::sync::Arc::new(edirstat_core::state::SharedState::new()),
                        None,
                        None,
                        false,
                    )))
                }),
            )
            .await;

        // Remove the loading text; on failure, leave a crash note instead.
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(()) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p>eDirStat failed to start — see the developer console for details.</p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}

#[cfg(target_family = "wasm")]
unsafe extern "C" {
    fn __wasm_call_ctors();
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    eprintln!(
        "edirstat-web is only meant to run in the browser; build it for wasm32 (see scripts/build_web.sh)."
    );
}
