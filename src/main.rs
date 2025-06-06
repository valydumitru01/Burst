#![allow(
    dead_code,
    unused_variables,
    clippy::too_many_arguments,
    clippy::unnecessary_wraps
)]

use std::io::Write;

mod gapi;
mod log;
mod window;

use crate::gapi::app::App as GraphicApp;
use crate::log::log::init_log;
use anyhow::Result;
use window::window::MyWindow;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;

fn main() -> Result<()> {
    init_log();
    // Window

    let event_loop = EventLoop::new()?;
    let window = MyWindow::new(&event_loop);

    // App
    let mut app = GraphicApp::new(&window)?;
    event_loop.run(move |event, elwt| {
        match event {
            // Request a redrawing when all events were processed.
            Event::AboutToWait => window.get().request_redraw(),
            Event::WindowEvent { event, .. } => match event {
                // Render a frame if our Vulkan app is not being destroyed.
                WindowEvent::RedrawRequested if !elwt.exiting() => app.render(&window).unwrap(),
                // Destroy our Vulkan app.
                WindowEvent::CloseRequested => {
                    elwt.exit();
                    app.destroy();
                }
                _ => {}
            },
            _ => {}
        }
    })?;

    Ok(())
}
