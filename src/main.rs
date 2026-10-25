mod app;
mod camera;
mod mesh;
mod renderer;
mod ui;
use app::{Action, AppState};
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
    let event_loop = winit::event_loop::EventLoop::<Instant>::with_user_event().build()?;
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .with_title("STL Viewer — O: open / drop an STL")
        .with_inner_size(1000, 760)
        .build(&event_loop);
    let renderer = renderer::Renderer::new(&display)?;
    let mut buffer = None;
    let mut app = AppState::default();
    let mut ui = ui::Ui::default();
    let mut egui =
        egui_glium::EguiGlium::new(egui::ViewportId::ROOT, &display, &window, &event_loop);
    let proxy = event_loop.create_proxy();
    egui.egui_ctx().set_request_repaint_callback(move |info| {
        if let Some(deadline) = Instant::now().checked_add(info.delay) {
            let _ = proxy.send_event(deadline);
        }
    });
    let mut repaint_at: Option<Instant> = None;
    let mut modifiers = winit::keyboard::ModifiersState::empty();
    let mut cursor = None;
    let mut left = false;
    let mut right = false;
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
                ui.error = Some(e.clone());
                eprintln!("{e}");
            }
        }
    }
    #[allow(deprecated)]
    event_loop.run(|event, target| {
        target.set_control_flow(winit::event_loop::ControlFlow::Wait);
        if let Some(deadline) = repaint_at {
            if Instant::now() >= deadline {
                window.request_redraw();
                repaint_at = None;
            } else {
                target.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(deadline));
            }
        }
        if let Event::UserEvent(deadline) = event {
            repaint_at = Some(repaint_at.map_or(deadline, |old| old.min(deadline)));
            if deadline <= Instant::now() {
                window.request_redraw();
            } else {
                target.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                    repaint_at.unwrap(),
                ));
            }
            return;
        }
        let Event::WindowEvent { event, .. } = event else {
            return;
        };
        let response = egui.on_event(&window, &event);
        if response.repaint && !matches!(event, WindowEvent::RedrawRequested) {
            window.request_redraw();
        }
        let blocked = response.consumed || ui.modal() || egui.egui_ctx().is_popup_open();
        let mut actions = Vec::new();
        let mut open = None;
        match event {
            WindowEvent::CloseRequested => target.exit(),
            WindowEvent::DroppedFile(path) => open = Some(path),
            WindowEvent::Resized(size) if size.width > 0 && size.height > 0 => {
                display.resize(size.into());
                window.request_redraw();
            }
            WindowEvent::ModifiersChanged(value) => modifiers = value.state(),
            WindowEvent::MouseInput { state, button, .. } => {
                let over_hud = cursor.is_some_and(|(x, y)| {
                    ui.hud_rect.is_some_and(|rect| {
                        rect.contains(
                            egui::pos2(x as f32, y as f32) / egui.egui_ctx().pixels_per_point(),
                        )
                    })
                });
                if state == ElementState::Pressed && (blocked || over_hud) {
                    return;
                }
                if button == MouseButton::Left {
                    left = state == ElementState::Pressed;
                }
                if button == MouseButton::Right {
                    right = state == ElementState::Pressed;
                }
            }
            WindowEvent::CursorLeft { .. } => {
                ui.cube.cancel();
                cursor = None;
                left = false;
                right = false;
            }
            WindowEvent::Focused(false) => {
                ui.cube.cancel();
                cursor = None;
                left = false;
                right = false;
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some((x, y)) = cursor {
                    let dx = (position.x - x) as f32;
                    let dy = (position.y - y) as f32;
                    if left {
                        app.camera.orbit(dx, dy);
                    }
                    if right {
                        let h = ui.viewport.height.max(1) as f32;
                        app.camera.pan[0] += dx / h * app.camera.vertical_span() as f32;
                        app.camera.pan[1] -= dy / h * app.camera.vertical_span() as f32;
                    }
                    if left || right {
                        window.request_redraw();
                    }
                }
                cursor = Some((position.x, position.y));
            }
            WindowEvent::MouseWheel { delta, .. } if !blocked => {
                let d = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 / 40.,
                };
                app.camera.zoom = (app.camera.zoom * (d * 0.12).exp()).clamp(0.02, 100.);
                window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed && !event.repeat =>
            {
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::Escape) => {
                        if ui.modal() {
                            ui.close_dialogs();
                        } else if egui.egui_ctx().is_popup_open() {
                            egui::Popup::close_all(egui.egui_ctx());
                        } else {
                            actions.push(Action::Exit);
                        }
                    }
                    PhysicalKey::Code(KeyCode::F1) if !egui.egui_ctx().wants_keyboard_input() => {
                        ui.help = true
                    }
                    PhysicalKey::Code(key)
                        if !ui.modal()
                            && !egui.egui_ctx().is_popup_open()
                            && !egui.egui_ctx().wants_keyboard_input() =>
                    {
                        match key {
                            KeyCode::KeyR => actions.push(Action::Reset),
                            KeyCode::KeyW => actions.push(Action::ToggleWireframe),
                            KeyCode::KeyO if modifiers.is_empty() || modifiers.control_key() => {
                                actions.push(Action::Open)
                            }
                            _ => {}
                        }
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
                egui.run(&window, |ctx| {
                    actions.extend(ui.draw(
                        ctx,
                        &mut app,
                        buffer.as_ref(),
                        (size.width, size.height),
                    ));
                });

                let mut frame = display.draw();
                frame.clear_color_and_depth((0.055, 0.065, 0.085, 1.), 1.);
                if let Some(buffer) = &buffer {
                    let viewport = ui.viewport;
                    let matrix = app.camera.matrix(viewport);
                    if let Err(e) =
                        renderer.draw(&mut frame, &buffer.buffer, matrix, viewport, app.wire)
                    {
                        eprintln!("Render failed: {e}");
                        target.exit();
                    }
                }
                egui.paint(&display, &mut frame);
                if let Err(e) = frame.finish() {
                    eprintln!("Present failed: {e}");
                    target.exit();
                }
            }
            _ => {}
        }
        for action in actions {
            match action {
                Action::Open => {
                    open = rfd::FileDialog::new()
                        .add_filter("STL model", &["stl", "STL"])
                        .pick_file()
                }
                Action::Exit => target.exit(),
                action => app.apply(action),
            }
            window.request_redraw();
        }
        if let Some(path) = open {
            match load(path) {
                Ok(b) => {
                    buffer = Some(b);
                    app.apply(Action::Reset);
                }
                Err(e) => {
                    window.set_title(&format!("STL Viewer — {e} — O to retry"));
                    ui.error = Some(e.clone());
                    eprintln!("{e}");
                }
            }
            window.request_redraw();
        }
    })?;
    Ok(())
}
