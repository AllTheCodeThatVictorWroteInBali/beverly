#[cfg(target_arch = "wasm32")]
mod browser;

include!(concat!(env!("OUT_DIR"), "/demos.rs"));

fn main() {
    #[cfg(target_arch = "wasm32")]
    let selected = web_sys::window()
        .and_then(|window| window.location().search().ok())
        .and_then(|query| web_sys::UrlSearchParams::new_with_str(&query).ok())
        .and_then(|params| params.get("demo"))
        .unwrap_or_else(|| "checkbox".into());
    #[cfg(not(target_arch = "wasm32"))]
    let selected = std::env::args().nth(1).unwrap_or_else(|| "checkbox".into());

    if !run_selected(&selected) {
        #[cfg(target_arch = "wasm32")]
        if let Some(screen) = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("loading-screen"))
        {
            screen.set_text_content(Some("This demo is unavailable."));
        }
        #[cfg(not(target_arch = "wasm32"))]
        eprintln!("Unknown demo: {selected}");
    }
}
