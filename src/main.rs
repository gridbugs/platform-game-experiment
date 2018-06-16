#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;

mod graphics;
mod glutin_window;

use gfx::Device;
use glutin::GlContext;
use graphics::quad::Renderer;
use glutin_window::GlutinWindow;

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

struct GraphicsTest;
impl<'a> graphics::quad::Update for &'a GraphicsTest {
    fn colour(&self) -> [f32; 3] {
        [0., 1., 0.]
    }
    fn position(&self) -> [f32; 2] {
        [100., 200.]
    }
    fn size(&self) -> [f32; 2] {
        [300., 100.]
    }
}

fn main() {
    let GlutinWindow {
        window,
        mut device,
        mut factory,
        render_target_view,
        mut events_loop,
        mut encoder,
        ..
    } = GlutinWindow::new(960, 640);

    let mut renderer =
        Renderer::new(render_target_view.clone(), &mut factory, &mut encoder);

    loop {
        encoder.clear(&render_target_view, [0.0, 0.0, 0.0, 1.0]);
        match process_input(&mut events_loop) {
            Some(ExternalEvent::Quit) => break,
            None => (),
        }
        renderer.update(&[GraphicsTest], &mut factory);
        renderer.encode(&mut encoder);
        encoder.flush(&mut device);
        window.swap_buffers().expect("Failed to swap buffers");
        device.cleanup();
    }
}
