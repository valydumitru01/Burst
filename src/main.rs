#![allow(
    dead_code,
    unused_variables,
    clippy::too_many_arguments,
    clippy::unnecessary_wraps
)]

mod gapi;
mod window;

use anyhow::Result;
use window::window::MyWindow;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;

fn main() -> Result<()> {
    pretty_env_logger::init();

    // Window

    let event_loop = EventLoop::new()?;
    let window = MyWindow::new(&event_loop);

    // App

    let mut app = unsafe { gapi::vulkan::App::create(&window)? };
    event_loop.run(move |event, elwt| {
        match event {
            // Request a redraw when all events were processed.
            Event::AboutToWait => window.get().request_redraw(),
            Event::WindowEvent { event, .. } => match event {
                // Render a frame if our Vulkan app is not being destroyed.
                WindowEvent::RedrawRequested if !elwt.exiting() => {
                    unsafe { app.render(&window) }.unwrap()
                }
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
