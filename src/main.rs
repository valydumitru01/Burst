use ::log::{debug, error};
use std::error::Error;

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
    if let Err(err) = run() {
        // Customize your error printing here
        error!("Oops! Something went wrong: {}", err);

        // You can even print source errors if you want
        let mut source = err.source();
        while let Some(cause) = source {
            error!("Caused by: {}", cause);
            source = cause.source();
        }

        std::process::exit(1);
    }
    Ok(())
}

fn run() -> Result<()> {
    init_log();
    // Window

    let event_loop = EventLoop::new()?;
    debug!("Creating Window...");
    let window = MyWindow::new(&event_loop);
    info_success!("Window Created!");

    // App
    debug!("Creating App...");
    let mut app = GraphicApp::new(&window)?;
    info_success!("App Created!");
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
