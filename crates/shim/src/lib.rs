//! tool_shim: moves values between the page and `tool_core::game`.
//!
//! Builder requirement: the shim decides nothing. Each function is one call
//! into the core and a conversion of its arguments or result. The core speaks
//! Q16.16 integers; the few values WebGL wants as floats (the matrix, the
//! uniforms, sound samples) are divided by the core's own `ONE` here.
//! `crates/core/tests/boundary.rs` fails if a branch, a loop or the `?`
//! operator appears here.

use tool_core::{content::Files, fx, game};
use wasm_bindgen::prelude::*;

const ONE: f32 = fx::ONE as f32;

fn floats(v: Vec<i32>) -> Vec<f32> {
    v.into_iter().map(|x| x as f32 / ONE).collect()
}

#[wasm_bindgen]
pub struct Game {
    inner: game::Game,
}

#[wasm_bindgen]
impl Game {
    #[allow(clippy::too_many_arguments)]
    #[wasm_bindgen(constructor)]
    pub fn new(
        menu: &str,
        levels: &str,
        band: &str,
        hints: &str,
        riffs: &str,
        copy: &str,
        scene: &str,
        masklings: &str,
        seed: u32,
    ) -> Result<Game, JsValue> {
        let files = Files {
            menu,
            levels,
            band,
            hints,
            riffs,
            copy,
            scene,
            masklings,
        };
        game::Game::new(&files, seed)
            .map(|inner| Game { inner })
            .map_err(|e| JsValue::from_str(&e))
    }

    /// CSS size and device pixel ratio in; drawing-buffer size out.
    pub fn resize(&mut self, css_w: u32, css_h: u32, dpr: f64) -> Vec<i32> {
        self.inner
            .resize(css_w as i32, css_h as i32, (dpr * 100.0) as i32)
            .to_vec()
    }

    /// The page's clock in milliseconds in; the sounds to play now out, as JSON.
    pub fn advance(&mut self, now_ms: f64) -> String {
        serde_json_list(self.inner.advance(now_ms as i64))
    }

    /// One named action; an empty string when done, else why not.
    pub fn act(&mut self, name: &str, value: &str) -> String {
        self.inner.act(name, value).err().unwrap_or_default()
    }

    pub fn ui(&self) -> String {
        self.inner.ui_json()
    }

    pub fn ui_version(&self) -> u32 {
        self.inner.ui_version()
    }

    pub fn instances(&mut self) -> Vec<i32> {
        self.inner.instances()
    }

    pub fn draws(&mut self) -> Vec<u32> {
        self.inner.draws()
    }

    pub fn mats(&self) -> Vec<f32> {
        floats(self.inner.mats())
    }

    pub fn uniforms(&self) -> Vec<f32> {
        floats(self.inner.uniforms())
    }

    pub fn clear_color(&self) -> Vec<f32> {
        floats(self.inner.clear_color())
    }

    pub fn mesh(&self, id: u32) -> Vec<i32> {
        self.inner.mesh(id)
    }
}

fn serde_json_list(sounds: Vec<&'static str>) -> String {
    game::sounds_json(&sounds)
}

#[wasm_bindgen]
pub fn numbers() -> String {
    game::numbers()
}

/// White noise for the page's sound, from the core's seeded generator.
#[wasm_bindgen]
pub fn noise(n: u32, seed: u32) -> Vec<f32> {
    floats(game::noise_samples(n, seed))
}
