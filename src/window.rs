use std::sync::Arc;

use anyhow::Result;
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

        self.state = Some(InnerApplication { window, surface });
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
                });

                output.present();
                // state.window.request_redraw();
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
