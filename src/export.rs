use crate::{
    app::{AppState, Display},
    camera::ViewportRect,
    mesh::Vertex,
    renderer::Renderer,
};
use glium::Surface;
use std::path::Path;
pub fn screenshot(
    display: &Display,
    renderer: &Renderer,
    buffer: &glium::VertexBuffer<Vertex>,
    app: &AppState,
    size: (u32, u32),
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let (width, height) = size;
    if width == 0 || height == 0 || width > 8192 || height > 8192 {
        return Err("Image dimensions must be between 1 and 8192".into());
    }
    let texture = glium::texture::Texture2d::empty(display, width, height)?;
    let depth = glium::texture::DepthTexture2d::empty(display, width, height)?;
    let mut frame =
        glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(display, &texture, &depth)?;
    let c = app.background.map(|v| v as f32 / 255.);
    frame.clear_color_and_depth((c[0], c[1], c[2], 1.), 1.);
    let viewport = ViewportRect {
        left: 0,
        bottom: 0,
        width,
        height,
    };
    renderer.draw(
        &mut frame,
        buffer,
        app.camera.matrix(viewport),
        viewport,
        app.wire,
        app.model_color,
    )?;
    let pixels: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    let bytes: Vec<u8> = pixels
        .into_iter()
        .rev()
        .flatten()
        .flat_map(|(r, g, b, a)| [r, g, b, a])
        .collect();
    let mut encoded = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut encoded, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.write_header()?.write_image_data(&bytes)?;
    }
    std::fs::write(path, encoded)?;
    Ok(())
}
