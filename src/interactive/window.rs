use std::sync::Arc;

use anyhow::Result;
use egui::FontDefinitions;
use egui_wgpu_backend::ScreenDescriptor;
use egui_winit_platform::{Platform, PlatformDescriptor};
use wgpu::{
    Color, CompositeAlphaMode, LoadOp, Operations, PresentMode, RenderPass,
    RenderPassColorAttachment, RenderPassDescriptor, StoreOp, Surface, SurfaceConfiguration,
    TextureUsages, TextureViewDescriptor,
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoopBuilder},
    window::{WindowAttributes, WindowId},
};

use crate::{gpu::Gpu, TEXTURE_FORMAT};

pub struct Window<'a> {
    app: Application<'a>,
}

struct Application<'a> {
    gpu: Gpu,
    attributes: WindowAttributes,
    render: Box<dyn Fn(&mut RenderPass)>,

    state: Option<InnerApplication<'a>>,
}

struct InnerApplication<'a> {
    window: Arc<winit::window::Window>,
    surface: Surface<'a>,

    egui_platform: Platform,
    egui_render: egui_wgpu_backend::RenderPass,
}

impl<'a> Window<'a> {
    pub fn run(mut self) -> Result<()> {
        let event_loop_builder = EventLoopBuilder::default().build()?;
        event_loop_builder.set_control_flow(ControlFlow::Wait);
        event_loop_builder.run_app(&mut self.app)?;
        Ok(())
    }
}

impl<'a> ApplicationHandler for Application<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(self.attributes.clone()).unwrap());
        let surface = self.gpu.instance.create_surface(window.clone()).unwrap();

        let size = window.inner_size();
        let egui_platform = egui_winit_platform::Platform::new(PlatformDescriptor {
            physical_width: size.width,
            physical_height: size.height,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });
        let egui_render = egui_wgpu_backend::RenderPass::new(&self.gpu.device, TEXTURE_FORMAT, 1);

        self.state = Some(InnerApplication {
            window,
            surface,
            egui_platform,
            egui_render,
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = &mut self.state else { return };
        if window_id != state.window.id() {
            return;
        }

        state.egui_platform.handle_event(&event);
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_size) => self.resize_surface(),
            WindowEvent::RedrawRequested => {
                let output = state.surface.get_current_texture().unwrap();

                self.gpu.dispatch(|encoder| {
                    let view = output
                        .texture
                        .create_view(&TextureViewDescriptor::default());

                    let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(Color::BLACK),
                                store: StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    (self.render)(&mut render_pass);
                    drop(render_pass);

                    state.egui_platform.begin_pass();
                    let ctx = state.egui_platform.context();
                    egui::Window::new("Ello").show(&ctx, |ui| {
                        ui.label("it works!?");
                    });

                    let egui_output = state.egui_platform.end_pass(Some(&state.window));
                    let egui_paint = ctx.tessellate(egui_output.shapes, ctx.pixels_per_point());

                    let size = state.window.inner_size();
                    let screen_descriptor = ScreenDescriptor {
                        physical_width: size.width,
                        physical_height: size.height,
                        scale_factor: state.window.scale_factor() as f32,
                    };

                    state
                        .egui_render
                        .add_textures(
                            &self.gpu.device,
                            &self.gpu.queue,
                            &egui_output.textures_delta,
                        )
                        .unwrap();
                    state.egui_render.update_buffers(
                        &self.gpu.device,
                        &self.gpu.queue,
                        &egui_paint,
                        &screen_descriptor,
                    );

                    state
                        .egui_render
                        .execute(encoder, &view, &egui_paint, &screen_descriptor, None)
                        .unwrap();
                });

                output.present();
                //  egui_rpass
                //     .remove_textures(tdelta)
                //     .expect("remove texture ok");
                state.window.request_redraw();
            }
            _ => {}
        }
    }
}

impl<'a> Application<'a> {
    fn resize_surface(&mut self) {
        let state = self.state.as_mut().unwrap();
        let size = state.window.inner_size();
        state.surface.configure(
            &self.gpu.device,
            &SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: TEXTURE_FORMAT,
                width: size.width,
                height: size.height,
                present_mode: PresentMode::AutoVsync,
                desired_maximum_frame_latency: 1,
                alpha_mode: CompositeAlphaMode::Opaque,
                view_formats: vec![],
            },
        );
    }
}

impl Gpu {
    pub fn create_window(
        &self,
        attributes: WindowAttributes,
        render: impl Fn(&mut RenderPass) + 'static,
    ) -> Window {
        Window {
            app: Application {
                gpu: self.clone(),
                attributes,
                render: Box::new(render),

                state: None,
            },
        }
    }
}
