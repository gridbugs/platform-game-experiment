#![feature(nonzero)]
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
mod loose_quad_tree;

use cgmath::vec2;
use game::GameState;
use gfx::Device;
use glutin::GlContext;
use glutin_window::GlutinWindow;
use graphics::quad::Renderer;

enum ExternalEvent {
    Quit,
}

fn process_input(events_loop: &mut glutin::EventsLoop) -> Option<ExternalEvent> {
    let mut external_event = None;

    events_loop.poll_events(|event| match event {
        glutin::Event::WindowEvent { event, .. } => match event {
            glutin::WindowEvent::CloseRequested => {
                external_event = Some(ExternalEvent::Quit);
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

    loop {
        encoder.clear(&render_target_view, [0.0, 0.0, 0.0, 1.0]);
        match process_input(&mut events_loop) {
            Some(ExternalEvent::Quit) => break,
            None => (),
        }
        game_state.update();
        renderer.update(game_state.renderer_updates(), &mut factory);
        renderer.encode(&mut encoder);
        encoder.flush(&mut device);
        window.swap_buffers().expect("Failed to swap buffers");
        device.cleanup();
    }
}
