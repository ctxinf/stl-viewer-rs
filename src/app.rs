use crate::mesh::Vertex;
pub type Display = glium::Display<glium::glutin::surface::WindowSurface>;
pub struct ModelState {
    pub buffer: glium::VertexBuffer<Vertex>,
    pub triangles: usize,
    pub center: [f64; 3],
    pub radius: f64,
    pub bounds: [[f64; 3]; 2],
}
