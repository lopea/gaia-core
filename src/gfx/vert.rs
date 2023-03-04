use bytemuck::{Zeroable, Pod};
use vulkano::impl_vertex;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Vertex {
    pub position: [f32; 4],
    pub tex_coord: [f32; 2],
}

impl_vertex!(Vertex, position, tex_coord);
