use anyhow::{Ok, Result};
use encase::ShaderType;
use tufa::{
    bindings::{IndexBuffer, UniformBuffer, VertexBuffer},
    export::{
        egui::Context,
        nalgebra::{Matrix4, Vector2, Vector3, Vector4},
        wgpu::{include_wgsl, RenderPass, ShaderStages},
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

#[derive(ShaderType, Default)]
struct Uniform {
    transformation: Matrix4<f32>,
}

struct App {
    camera: PerspectiveCamera,
    uniform: UniformBuffer<Uniform>,
    vertex: VertexBuffer<Vertex>,
    index: IndexBuffer,
    render: RenderPipeline,
}

fn main() -> Result<()> {
    let gpu = Gpu::new()?;

    let uniform = gpu.create_uniform(&Uniform::default());
    let vertex = gpu.create_vertex(&[
        Vertex::new(Vector4::new(0.0, 0.5, 0.0, 1.0), Vector2::new(0.0, 1.0)),
        Vertex::new(Vector4::new(-0.5, -0.5, 0.0, 1.0), Vector2::new(0.0, 0.0)),
        Vertex::new(Vector4::new(0.5, -0.5, 0.0, 1.0), Vector2::new(1.0, 0.0)),
    ]);
    let index = gpu.create_index(&[0, 1, 2]);
    let render = gpu
        .render_pipeline(include_wgsl!("shader.wgsl"))
        .bind(&uniform, ShaderStages::VERTEX)
        .finish();

    gpu.create_window(
        WindowAttributes::default().with_title("Simple 3d"),
        App {
            camera: PerspectiveCamera::default().with_position(-Vector3::z()),
            uniform,
            vertex,
            index,
            render,
        },
    )
    .run()?;

    Ok(())
}

impl Interactive for App {
    fn render(&mut self, gcx: GraphicsCtx, render_pass: &mut RenderPass) {
        gcx.window.set_cursor_grab(CursorGrabMode::Locked).unwrap();
        gcx.window.set_cursor_visible(false);

        let size = gcx.window.inner_size();
        let aspect = size.width as f32 / size.height as f32;

        let transformation = self.camera.view_projection(aspect);
        self.uniform.upload(&Uniform { transformation });

        self.render
            .draw(render_pass, &self.index, &self.vertex, 0..3);
    }

    fn ui(&mut self, _gcx: GraphicsCtx, ctx: &Context) {
        self.camera.update(ctx);
    }

    fn device_event(&mut self, _gcx: GraphicsCtx, _device_id: DeviceId, event: &DeviceEvent) {
        self.camera.device_event(event);
    }
}
