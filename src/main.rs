mod gfx;
use gfx::{render_system::RenderInstance, viewport::Viewport};
use winit::{event_loop::EventLoop, event::{Event,WindowEvent}};
fn main() {
    let event_loop = EventLoop::new();

    let _renderer = RenderInstance::new(&event_loop);
    let _viewport = Viewport::new(&event_loop, _renderer, 1024, 768);

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();
        
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                control_flow.set_exit();
            },
            _ => (),
        }
    });

}

