//! Game of Trains client.
//!
//! A `macroquad` renderer for the `train-core` engine. Runs natively (a desktop
//! window) and compiles to WebAssembly for the PWA. All game logic lives in
//! `train-core`; this crate only renders state and feeds in taps.

mod app;
mod audio;
mod fx;
mod theme;
mod view;

use app::App;
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Game of Trains".to_owned(),
        window_width: 480,
        window_height: 800,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = App::new();
    loop {
        app.update();
        app.draw();
        next_frame().await;
    }
}
