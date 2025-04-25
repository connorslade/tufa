use std::sync::Arc;

use anyhow::Result;
use egui_wgpu::ScreenDescriptor;
use wgpu::{
    Color, CompositeAlphaMode, LoadOp, Operations, PresentMode, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, StoreOp, Surface, SurfaceConfiguration,
    Texture, TextureDescriptor, TextureDimension, TextureUsages, TextureViewDescriptor,
};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoopBuilder},
    window::{WindowAttributes, WindowId},
};

use crate::{gpu::Gpu, DEPTH_TEXTURE_FORMAT, TEXTURE_FORMAT};

use super::{egui::Egui, GraphicsCtx, Interactive};

pub struct Window<'a, T> {
    app: Application<'a, T>,
}

struct Application<'a, T> {
    gpu: Gpu,
    attributes: WindowAttributes,
    state: Option<InnerApplication<'a>>,

    interactive: T,
}

struct InnerApplication<'a> {
    window: Arc<winit::window::Window>,
    surface: Surface<'a>,
    depth_texture: Texture,
    egui: Egui,
}

impl<T: Interactive> Window<'_, T> {
    pub fn run(mut self) -> Result<()> {
        let event_loop_builder = EventLoopBuilder::default().build()?;
        event_loop_builder.set_control_flow(ControlFlow::Wait);
        event_loop_builder.run_app(&mut self.app)?;
        Ok(())
    }
}

impl<T: Interactive> ApplicationHandler for Application<'_, T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(self.attributes.clone()).unwrap());
        let surface = self.gpu.instance.create_surface(window.clone()).unwrap();
        let egui = Egui::new(&self.gpu.device, TEXTURE_FORMAT, None, 1, &window);

        let window_size = window.inner_size();
        let depth_texture = self.gpu.device.create_texture(&TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: window_size.width,
                height: window_size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: DEPTH_TEXTURE_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let gcx = GraphicsCtx {
            gpu: &self.gpu,
            window: &window,
        };
        self.interactive.init(gcx);

        self.state = Some(InnerApplication {
            window,
            surface,
            depth_texture,
            egui,
        });
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let Some(state) = &mut self.state else { return };
        self.interactive.device_event(
            GraphicsCtx {
                gpu: &self.gpu,
                window: &state.window,
            },
            device_id,
            &event,
        );
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

        let gcx = GraphicsCtx {
            gpu: &self.gpu,
            window: &state.window,
        };
        self.interactive.window_event(gcx.clone(), &event);
        state.egui.handle_input(&state.window, &event);
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_size) => self.resize_surface(),
            WindowEvent::RedrawRequested => {
                let output = state.surface.get_current_texture().unwrap();

                self.gpu.immediate_dispatch(|encoder| {
                    let view = output
                        .texture
                        .create_view(&TextureViewDescriptor::default());
                    let depth_view = state
                        .depth_texture
                        .create_view(&TextureViewDescriptor::default());

                    {
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
                            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                                view: &depth_view,
                                depth_ops: Some(Operations {
                                    load: LoadOp::Clear(1.0),
                                    store: StoreOp::Store,
                                }),
                                stencil_ops: None,
                            }),
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });
                        self.interactive.render(gcx.clone(), &mut render_pass);
                    }

                    {
                        state.egui.begin_frame(&state.window);
                        self.interactive.ui(gcx, state.egui.context());

                        let size = state.window.inner_size();
                        state.egui.end_frame_and_draw(
                            &self.gpu.device,
                            &self.gpu.queue,
                            encoder,
                            &state.window,
                            &view,
                            ScreenDescriptor {
                                size_in_pixels: [size.width, size.height],
                                pixels_per_point: state.window.scale_factor() as f32,
                            },
                        );
                    }
                });

                output.present();
                state.window.request_redraw();
            }
            _ => {}
        }
    }
}

impl<T> Application<'_, T> {
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
        state.depth_texture = self.gpu.device.create_texture(&TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: DEPTH_TEXTURE_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
    }
}

impl Gpu {
    pub fn create_window<T: Interactive>(
        &self,
        attributes: WindowAttributes,
        interactive: T,
    ) -> Window<T> {
        Window {
            app: Application {
                gpu: self.clone(),
                attributes,
                state: None,

                interactive,
            },
        }
    }
}
