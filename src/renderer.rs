use crate::{app::Display, camera::ViewportRect, mesh::Vertex};
use glium::{Surface, uniform};
pub struct Renderer {
    program: glium::Program,
}
impl Renderer {
    pub fn new(display: &Display) -> Result<Self, glium::program::ProgramCreationError> {
        let program = glium::Program::from_source(
            display,
            "#version 140\nin vec3 position; uniform mat4 matrix; out vec3 pos; void main(){pos=position; gl_Position=matrix*vec4(position,1.0);}",
            "#version 140\nin vec3 pos; out vec4 color; void main(){vec3 n=normalize(cross(dFdx(pos),dFdy(pos))); float l=0.28+0.72*abs(dot(n,normalize(vec3(0.4,-0.6,1.0)))); color=vec4(vec3(0.32,0.68,0.88)*l,1.0);}",
            None,
        )?;
        Ok(Self { program })
    }
    pub fn draw<S: Surface>(
        &self,
        frame: &mut S,
        buffer: &glium::VertexBuffer<Vertex>,
        matrix: glam::Mat4,
        v: ViewportRect,
        wire: bool,
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
            &uniform! {matrix:matrix.to_cols_array_2d()},
            &params,
        )
    }
}
