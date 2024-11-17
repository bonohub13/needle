use anyhow::Result;
use needle_core::{NeedleError, State, TimeFormat};
use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

const APP_NAME: &'static str = "needle";

fn main() -> Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title(APP_NAME)
        .build(&event_loop)?;
    let frame_cap = std::time::Duration::from_secs_f64(1.0 / 30.0); // 30 fps
    let mut app = pollster::block_on(State::new(&window, TimeFormat::HourMinSec, 5.0))?;
    let mut next_frame = std::time::Instant::now();

    event_loop.run(move |event, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == app.window().id() => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => control_flow.exit(),
            WindowEvent::Resized(physical_size) => {
                // Resize Window
                app.resize(physical_size);
            }
            WindowEvent::RedrawRequested => {
                // Main Render Loop
                app.window().request_redraw();
                match app.update() {
                    Ok(_) => (),
                    Err(err) => {
                        log::error!("Failed to update frame: {}", err);

                        control_flow.exit();
                    }
                };

                match app.render() {
                    Ok(_) => {
                        next_frame += frame_cap;
                        std::thread::sleep(next_frame - std::time::Instant::now());
                    }
                    Err(err) => match err {
                        NeedleError::Lost | NeedleError::Outdated => app.resize(&app.size()),
                        NeedleError::OutOfMemory | NeedleError::RemovedFromAtlas => {
                            log::error!("OutOfMemory");
                            control_flow.exit();
                        }
                        NeedleError::Timeout => {
                            log::warn!("Surface Timeout")
                        }
                        _ => (),
                    },
                }
            }
            _ => {}
        },
        _ => {}
    })?;

    Ok(())
}
