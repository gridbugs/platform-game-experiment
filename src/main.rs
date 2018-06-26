#![feature(nonzero)]
extern crate best;
extern crate cgmath;
extern crate fnv;
#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;

mod aabb;
mod game;
mod glutin_window;
mod graphics;
mod line_segment;
mod loose_quad_tree;
mod shape;

use cgmath::vec2;
use game::{GameState, InputModel};
use gfx::Device;
use glutin::GlContext;
use glutin_window::GlutinWindow;
use graphics::Renderer;

enum ExternalEvent {
    Quit,
    Reset,
}

fn process_input(
    events_loop: &mut glutin::EventsLoop,
    input_model: &mut InputModel,
) -> Option<ExternalEvent> {
    let mut external_event = None;

    events_loop.poll_events(|event| match event {
        glutin::Event::WindowEvent { event, .. } => match event {
            glutin::WindowEvent::CloseRequested => {
                external_event = Some(ExternalEvent::Quit);
            }
            glutin::WindowEvent::KeyboardInput { input, .. } => {
                if let Some(virtual_keycode) = input.virtual_keycode {
                    match input.state {
                        glutin::ElementState::Pressed => match virtual_keycode {
                            glutin::VirtualKeyCode::Return => {
                                external_event = Some(ExternalEvent::Reset)
                            }
                            glutin::VirtualKeyCode::Left => input_model.set_left(1.),
                            glutin::VirtualKeyCode::Right => input_model.set_right(1.),
                            glutin::VirtualKeyCode::Up => input_model.set_up(1.),
                            glutin::VirtualKeyCode::Down => input_model.set_down(1.),
                            _ => (),
                        },
                        glutin::ElementState::Released => match virtual_keycode {
                            glutin::VirtualKeyCode::Left => input_model.set_left(0.),
                            glutin::VirtualKeyCode::Right => input_model.set_right(0.),
                            glutin::VirtualKeyCode::Up => input_model.set_up(0.),
                            glutin::VirtualKeyCode::Down => input_model.set_down(0.),
                            _ => (),
                        },
                    }
                }
            }
            _ => (),
        },
        _ => (),
    });

    external_event
}

fn main() {
    let width = 960;
    let height = 640;
    let GlutinWindow {
        window,
        mut device,
        mut factory,
        render_target_view,
        mut events_loop,
        mut encoder,
        ..
    } = GlutinWindow::new(width, height);

    let mut renderer =
        Renderer::new(render_target_view.clone(), &mut factory, &mut encoder);

    let mut game_state = GameState::new(vec2(width as f32, height as f32));
    game_state.init_demo();

    let mut input_model = InputModel::default();

    loop {
        encoder.clear(&render_target_view, [0.0, 0.0, 0.0, 1.0]);
        match process_input(&mut events_loop, &mut input_model) {
            Some(ExternalEvent::Quit) => break,
            Some(ExternalEvent::Reset) => game_state.init_demo(),
            None => (),
        }
        game_state.update(&input_model);
        {
            let mut quad_writer = renderer.quad_writer(&mut factory);
            let mut quad_writer_iter = quad_writer.iter_mut();
            for common in game_state.common_iter() {
                if let Some(quad) = quad_writer_iter.next() {
                    quad.position_of_top_left_in_pixels = common.top_left.into();
                    quad.dimensions_in_pixels = common.shape.dimensions().into();
                    quad.colour = common.colour;
                }
            }
        }
        renderer.encode(&mut encoder);
        encoder.flush(&mut device);
        window.swap_buffers().expect("Failed to swap buffers");
        device.cleanup();
    }
}
