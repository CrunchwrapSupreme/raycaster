mod map;
mod render;

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use std::time::{Instant};
use std::{thread, time};

const WIDTH: u32 = render::RENDER_WIDTH;
const HEIGHT: u32 = render::RENDER_HEIGHT;
const FRAME_SLEEP: f64 = 1.0 / 60.0;

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Raycast Renderer")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(render::RENDER_WIDTH, render::RENDER_HEIGHT, surface_texture)?
    };
    let mut world = render::RenderState::new();
    window.set_cursor_grab(false).unwrap();
    window.set_cursor_visible(false);
    let mut time_start = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.render(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            let delta_t = Instant::now().duration_since(time_start);
            let mut delta_t = delta_t.as_secs_f64();
            time_start = Instant::now();
            if FRAME_SLEEP > delta_t {
                thread::sleep(time::Duration::from_secs_f64(FRAME_SLEEP - delta_t));
                delta_t = FRAME_SLEEP;
            }
            world.update(&input, delta_t);
            window.request_redraw();
        }
    });
}
