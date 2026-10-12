#[cfg(target_arch = "wasm32")]
pub use crate::browser::DemoAppExt;

#[cfg(not(target_arch = "wasm32"))]
use bevy::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
pub trait DemoAppExt {
    fn run_demo(&mut self) -> AppExit;
}

#[cfg(not(target_arch = "wasm32"))]
impl DemoAppExt for App {
    fn run_demo(&mut self) -> AppExit {
        self.run()
    }
}
