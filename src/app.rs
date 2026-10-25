use crate::mesh::Vertex;
pub type Display = glium::Display<glium::glutin::surface::WindowSurface>;
pub struct ModelState {
    pub buffer: glium::VertexBuffer<Vertex>,
    pub triangles: usize,
    pub center: [f64; 3],
    pub radius: f64,
    pub bounds: [[f64; 3]; 2],
}

pub enum Action {
    Open,
    Reset,
    SetView(crate::camera::View),
    ToggleWireframe,
    Exit,
}
pub struct AppState {
    pub camera: crate::camera::Camera,
    pub wire: bool,
    pub show_cube: bool,
    pub show_scale: bool,
}
impl Default for AppState {
    fn default() -> Self {
        Self {
            camera: Default::default(),
            wire: false,
            show_cube: true,
            show_scale: true,
        }
    }
}
impl AppState {
    pub fn apply(&mut self, action: Action) {
        match action {
            Action::Reset => self.camera = Default::default(),
            Action::SetView(view) => self.camera.set_view(view),
            Action::ToggleWireframe => self.wire = !self.wire,
            Action::Open | Action::Exit => {}
        }
    }
}
