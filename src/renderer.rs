use crate::{app::Display, camera::ViewportRect, mesh::Vertex};
use glium::{Surface, uniform};
pub struct Renderer {
    program: glium::Program,
}
impl Renderer {
    pub fn new(display: &Display) -> Result<Self, glium::program::ProgramCreationError> {
        // macOS exposes OpenGL 3.2+ core contexts with GLSL 1.50.
        let version = if cfg!(target_os = "macos") {
            "150"
        } else {
            "140"
        };
        let vertex = format!(
            "#version {version}\nin vec3 position; uniform mat4 matrix; out vec3 pos; void main(){{pos=position; gl_Position=matrix*vec4(position,1.0);}}"
        );
        let fragment = format!(
            "#version {version}\nin vec3 pos; uniform vec3 model_color; out vec4 color; void main(){{vec3 n=normalize(cross(dFdx(pos),dFdy(pos))); float l=0.28+0.72*abs(dot(n,normalize(vec3(0.4,-0.6,1.0)))); color=vec4(model_color*l,1.0);}}"
        );
        let program = glium::Program::from_source(display, &vertex, &fragment, None)?;
        Ok(Self { program })
    }
    pub fn draw<S: Surface>(
        &self,
        frame: &mut S,
        buffer: &glium::VertexBuffer<Vertex>,
        matrix: glam::Mat4,
        v: ViewportRect,
        wire: bool,
        model_color: [u8; 3],
    ) -> Result<(), glium::DrawError> {
        let rect = glium::Rect {
            left: v.left,
            bottom: v.bottom,
            width: v.width,
            height: v.height,
        };
        let params = glium::DrawParameters {
            viewport: Some(rect),
            scissor: Some(rect),
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            polygon_mode: if wire {
                glium::draw_parameters::PolygonMode::Line
            } else {
                glium::draw_parameters::PolygonMode::Fill
            },
            ..Default::default()
        };
        frame.draw(
            buffer,
            glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
            &self.program,
            &uniform! {matrix:matrix.to_cols_array_2d(), model_color: model_color.map(|v| v as f32 / 255.)},
            &params,
        )
    }
}
