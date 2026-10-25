mod app;
mod camera;
mod mesh;
mod renderer;
use camera::{Camera, ViewportRect};
use mesh::Vertex;
glium::implement_vertex!(Vertex, position);

use glium::{
    Surface,
    winit::{
        self,
        event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent},
        keyboard::{KeyCode, PhysicalKey},
    },
};
use std::{path::PathBuf, time::Instant};

fn open_mesh(path: &std::path::Path) -> Result<mesh::Mesh, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    // Read-only mapping is dropped after parsing. The source must not be
    // modified or truncated concurrently while opening it.
    let data = unsafe { memmap2::Mmap::map(&file) }.map_err(|e| e.to_string())?;
    mesh::Mesh::parse(&data)
}

fn main() {
    if let Err(e) = run() {
        eprintln!("STL Viewer: {e}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let first = args.next();
    if first.as_deref() == Some(std::ffi::OsStr::new("--help")) {
        println!(
            "STL Viewer {}\nUsage: stl-viewer-linux [FILE.stl]\nDrag: orbit | Right drag: pan | Wheel: zoom | O: open | R: reset | W: wireframe | Esc: exit\n--info FILE: parse and report without opening a window",
            env!("CARGO_PKG_VERSION")
        );
        return Ok(());
    }
    if first.as_deref() == Some(std::ffi::OsStr::new("--info")) {
        let path = args.next().ok_or("--info requires a file")?;
        let start = Instant::now();
        let mesh = open_mesh(std::path::Path::new(&path))?;
        println!(
            "triangles={} radius={} center={:?} parse_ms={:.3}",
            mesh.vertices.len() / 3,
            mesh.radius,
            mesh.center,
            start.elapsed().as_secs_f64() * 1000.
        );
        return Ok(());
    }
    if args.next().is_some() {
        return Err("Expected one STL file; use --help".into());
    }
    let event_loop = winit::event_loop::EventLoop::new()?;
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .with_title("STL Viewer — O: open / drop an STL")
        .with_inner_size(1000, 760)
        .build(&event_loop);
    let renderer = renderer::Renderer::new(&display)?;
    let mut buffer = None;
    let mut camera = Camera::default();
    let mut cursor = None;
    let mut left = false;
    let mut right = false;
    let mut wire = false;
    let load = |path: PathBuf| -> Result<app::ModelState, String> {
        let start = Instant::now();
        let mesh = open_mesh(&path)?;
        let buffer =
            glium::VertexBuffer::immutable(&display, &mesh.vertices).map_err(|e| e.to_string())?;
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        window.set_title(&format!(
            "{name} — {} triangles — {:.0} ms | O open · R reset · W wireframe",
            mesh.vertices.len() / 3,
            start.elapsed().as_secs_f64() * 1000.
        ));
        Ok(app::ModelState {
            buffer,
            triangles: mesh.vertices.len() / 3,
            center: mesh.center,
            radius: mesh.radius,
            bounds: mesh.bounds,
        })
    };
    if let Some(path) = first {
        match load(path.into()) {
            Ok(b) => buffer = Some(b),
            Err(e) => {
                window.set_title(&format!("STL Viewer — {e} — O to open"));
                eprintln!("{e}");
            }
        }
    }
    #[allow(deprecated)]
    event_loop.run(|event, target| {
        target.set_control_flow(winit::event_loop::ControlFlow::Wait);
        let Event::WindowEvent { event, .. } = event else {
            return;
        };
        let mut open = None;
        match event {
            WindowEvent::CloseRequested => target.exit(),
            WindowEvent::DroppedFile(path) => open = Some(path),
            WindowEvent::Resized(size) if size.width > 0 && size.height > 0 => {
                display.resize(size.into());
                window.request_redraw();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left {
                    left = state == ElementState::Pressed;
                }
                if button == MouseButton::Right {
                    right = state == ElementState::Pressed;
                }
            }
            WindowEvent::CursorLeft { .. } => {
                cursor = None;
                left = false;
                right = false;
            }
            WindowEvent::Focused(false) => {
                left = false;
                right = false;
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some((x, y)) = cursor {
                    let dx = (position.x - x) as f32;
                    let dy = (position.y - y) as f32;
                    if left {
                        camera.orbit(dx, dy);
                    }
                    if right {
                        let h = window.inner_size().height.max(1) as f32;
                        camera.pan[0] += dx / h * 2.8 / camera.zoom;
                        camera.pan[1] -= dy / h * 2.8 / camera.zoom;
                    }
                    if left || right {
                        window.request_redraw();
                    }
                }
                cursor = Some((position.x, position.y));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let d = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 / 40.,
                };
                camera.zoom = (camera.zoom * (d * 0.12).exp()).clamp(0.02, 100.);
                window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed && !event.repeat =>
            {
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::Escape) => target.exit(),
                    PhysicalKey::Code(KeyCode::KeyR) => {
                        camera = Camera::default();
                    }
                    PhysicalKey::Code(KeyCode::KeyW) => wire = !wire,
                    PhysicalKey::Code(KeyCode::KeyO) => {
                        open = rfd::FileDialog::new()
                            .add_filter("STL model", &["stl", "STL"])
                            .pick_file()
                    }
                    _ => {}
                }
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let size = window.inner_size();
                if size.width == 0 || size.height == 0 {
                    return;
                }
                let mut frame = display.draw();
                frame.clear_color_and_depth((0.055, 0.065, 0.085, 1.), 1.);
                if let Some(buffer) = &buffer {
                    let viewport = ViewportRect {
                        left: 0,
                        bottom: 0,
                        width: size.width,
                        height: size.height,
                    };
                    let matrix = camera.matrix(viewport);
                    if let Err(e) =
                        renderer.draw(&mut frame, &buffer.buffer, matrix, viewport, wire)
                    {
                        eprintln!("Render failed: {e}");
                        target.exit();
                    }
                }
                if let Err(e) = frame.finish() {
                    eprintln!("Present failed: {e}");
                    target.exit();
                }
            }
            _ => {}
        }
        if let Some(path) = open {
            match load(path) {
                Ok(b) => {
                    buffer = Some(b);
                    camera = Camera::default();
                }
                Err(e) => {
                    window.set_title(&format!("STL Viewer — {e} — O to retry"));
                    eprintln!("{e}");
                }
            }
            window.request_redraw();
        }
    })?;
    Ok(())
}
