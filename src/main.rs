mod gfx;
use gfx::{render_system::RenderInstance, viewport::Viewport};
use winit::{event_loop::EventLoop, event::{Event,WindowEvent}};
fn main() {
    let mut event_loop = EventLoop::new();

    let _renderer = RenderInstance::new(&event_loop);
    let mut _viewport = Viewport::new(&event_loop, _renderer, 1024, 768);


    _viewport.run(&mut event_loop);
}

