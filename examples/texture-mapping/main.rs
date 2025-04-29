use std::time::Instant;

use anyhow::{Ok, Result};
use encase::ShaderType;
use tufa::{
    bindings::{texture::format::Rgba8, IndexBuffer, UniformBuffer, VertexBuffer},
    export::{
        egui::{Context, Key, Window},
        nalgebra::{Matrix4, Vector2, Vector3, Vector4},
        wgpu::{include_wgsl, FilterMode, RenderPass, ShaderStages},
        winit::{
            event::{DeviceEvent, DeviceId},
            window::{CursorGrabMode, WindowAttributes},
        },
    },
    gpu::Gpu,
    interactive::{GraphicsCtx, Interactive},
    misc::camera::PerspectiveCamera,
    pipeline::render::{RenderPipeline, Vertex},
};

struct App {
    start: Instant,
    camera: PerspectiveCamera,
    camera_active: bool,
    perspective_correction: bool,
    pipeline: RenderPipeline,

    uniform: UniformBuffer<Uniform>,
    vertex: VertexBuffer<Vertex>,
    index: IndexBuffer,
}

#[derive(Default, ShaderType)]
struct Uniform {
    transform: Matrix4<f32>,
    flags: u32,
}

fn main() -> Result<()> {
    let gpu = Gpu::new()?;

    let sampler = gpu.create_sampler(FilterMode::Nearest);
    let texture = gpu.create_texture_2d::<Rgba8>(Vector2::repeat(64));
    let image = image::load_from_memory(include_bytes!("brick.png")).unwrap();
    texture.upload(&image.into_rgba8());

    let uniform = gpu.create_uniform(&Uniform::default())?;
    let pipeline = gpu
        .render_pipeline(include_wgsl!("shader.wgsl"))
        .bind(&uniform, ShaderStages::VERTEX_FRAGMENT)
        .bind(&texture, ShaderStages::FRAGMENT)
        .bind(&sampler, ShaderStages::FRAGMENT)
        .finish();

    let vertex = gpu
        .create_vertex(&[
            Vertex::new(Vector4::new(-1.0, -1.0, 0.0, 1.0), Vector2::new(0.0, 0.0)),
            Vertex::new(Vector4::new(-1.0, 1.0, 0.0, 1.0), Vector2::new(0.0, 1.0)),
            Vertex::new(Vector4::new(1.0, 1.0, 0.0, 1.0), Vector2::new(1.0, 1.0)),
            Vertex::new(Vector4::new(1.0, -1.0, 0.0, 1.0), Vector2::new(1.0, 0.0)),
        ])
        .unwrap();
    let index = gpu.create_index(&[0, 1, 2, 2, 3, 0]);

    gpu.create_window(
        WindowAttributes::default(),
        App {
            start: Instant::now(),
            camera: PerspectiveCamera::default().with_position(Vector3::z() * -2.0),
            camera_active: true,
            perspective_correction: true,
            pipeline,

            uniform,
            vertex,
            index,
        },
    )
    .run()?;

    Ok(())
}

impl Interactive for App {
    fn render(&mut self, gcx: GraphicsCtx, render_pass: &mut RenderPass) {
        if self.camera_active {
            gcx.window.set_cursor_grab(CursorGrabMode::Locked).unwrap();
        } else {
            gcx.window.set_cursor_grab(CursorGrabMode::None).unwrap();
        }

        let t = self.start.elapsed().as_secs_f32();
        let model = Matrix4::new_rotation(Vector3::new(t.sin(), t.cos(), 0.0) * 0.5);

        let window = gcx.window.inner_size();
        let aspect = window.width as f32 / window.height as f32;
        self.uniform.upload(&Uniform {
            transform: self.camera.view_projection(aspect) * model,
            flags: self.perspective_correction as u32,
        });
        self.pipeline
            .draw(render_pass, &self.index, &self.vertex, 0..6);
    }

    fn ui(&mut self, _gcx: GraphicsCtx, ctx: &Context) {
        self.camera_active ^= ctx.input(|i| i.key_pressed(Key::Escape));

        Window::new("Texture Mapping").show(ctx, |ui| {
            ui.checkbox(&mut self.perspective_correction, "Perspective Correction");
            self.camera.ui(ui, "Camera");
        });

        if self.camera_active {
            self.camera.update(ctx);
        }
    }

    fn device_event(&mut self, _gcx: GraphicsCtx, _device_id: DeviceId, event: &DeviceEvent) {
        if self.camera_active {
            self.camera.device_event(event);
        }
    }
}
