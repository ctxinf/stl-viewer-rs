use crate::mesh::Vertex;
pub type Display = glium::Display<glium::glutin::surface::WindowSurface>;
pub struct ModelState {
    pub screenshot_name: std::ffi::OsString,
    pub buffer: glium::VertexBuffer<Vertex>,
    pub triangles: usize,
    pub load_time: std::time::Duration,
    pub center: [f64; 3],
    pub radius: f64,
    pub bounds: [[f64; 3]; 2],
}

pub enum Action {
    Open,
    Screenshot,
    Reset,
    SetView(crate::camera::View),
    ToggleWireframe,
    Exit,
}
pub struct AppState {
    pub camera: crate::camera::Camera,
    pub wire: bool,
    pub background: [u8; 3],
    pub model_color: [u8; 3],
    pub show_cube: bool,
    pub show_zoom: bool,
}
impl Default for AppState {
    fn default() -> Self {
        Self {
            camera: Default::default(),
            wire: false,
            background: crate::settings::load("background", crate::settings::DEFAULT),
            model_color: crate::settings::load("model", crate::settings::DEFAULT_MODEL),
            show_cube: true,
            show_zoom: true,
        }
    }
}
impl AppState {
    pub fn apply(&mut self, action: Action) {
        match action {
            Action::Reset => self.camera = Default::default(),
            Action::SetView(view) => self.camera.set_view(view),
            Action::ToggleWireframe => self.wire = !self.wire,
            Action::Open | Action::Screenshot | Action::Exit => {}
        }
    }
}
