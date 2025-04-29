use wgpu::{
    Color, LoadOp, Operations, RenderPass, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, StoreOp,
};

use crate::{
    bindings::{
        texture::format::{Depth, Rgba8},
        Texture,
    },
    gpu::Gpu,
};

impl Gpu {
    pub fn render_pass(
        &self,
        texture: &Texture<Rgba8>,
        depth: &Texture<Depth>,
        callback: impl FnOnce(&mut RenderPass),
    ) {
        self.immediate_dispatch(|encoder| {
            let manager = &self.binding_manager;

            let view = manager.get_resource(texture.id);
            let view = view.expect_texture_view();

            let depth = manager.get_resource(depth.id);
            let depth = depth.expect_texture_view();

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
                    view: &depth,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            callback(&mut render_pass);
        });
    }
}
