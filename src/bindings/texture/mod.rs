use std::marker::PhantomData;

use format::TextureFormat;
use nalgebra::{Vector2, Vector3};
use wgpu::{
    BindingType, Extent3d, Origin3d, TexelCopyBufferInfo, TexelCopyBufferLayout,
    TexelCopyTextureInfo, TextureAspect, TextureDescriptor, TextureDimension, TextureSampleType,
    TextureUsages, TextureViewDescriptor, TextureViewDimension,
};

use crate::{gpu::Gpu, misc::ids::TextureId};

use super::{buffer::BufferBinding, Bindable, BindableResourceId};

pub mod format;
mod sampler;
pub use sampler::Sampler;

pub struct Texture<Format: TextureFormat> {
    gpu: Gpu,

    pub(crate) id: TextureId,
    texture: wgpu::Texture,
    size: Vector3<u32>,

    _format: PhantomData<Format>,
}

impl<Format: TextureFormat> Texture<Format> {
    pub fn upload(&self, data: &[u8]) {
        assert_eq!(
            data.len(),
            self.size.iter().copied().product::<u32>() as usize * 4
        );

        self.gpu.queue.write_texture(
            self.texture.as_image_copy(),
            data,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(self.size.x * 4),
                rows_per_image: Some(self.size.y),
            },
            Extent3d {
                width: self.size.x,
                height: self.size.y,
                depth_or_array_layers: self.size.z,
            },
        );
    }

    pub fn copy_to_buffer<T: BufferBinding>(&self, buffer: &T) {
        let buffer = self.gpu.binding_manager.get_resource(buffer.get_id());
        let buffer = buffer.expect_buffer();

        self.gpu.immediate_dispatch(|encoder| {
            encoder.copy_texture_to_buffer(
                TexelCopyTextureInfo {
                    texture: &self.texture,
                    mip_level: 0,
                    origin: Origin3d::ZERO,
                    aspect: TextureAspect::All,
                },
                TexelCopyBufferInfo {
                    buffer,
                    layout: TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(self.size.x * 4),
                        rows_per_image: Some(self.size.y),
                    },
                },
                Extent3d {
                    width: self.size.x,
                    height: self.size.y,
                    depth_or_array_layers: self.size.z,
                },
            );
        });
    }
}

impl Gpu {
    pub fn create_texture_2d<Format: TextureFormat>(&self, size: Vector2<u32>) -> Texture<Format> {
        let texture = self.device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: Format::as_format(),
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let id = TextureId::new();
        let view = texture.create_view(&TextureViewDescriptor::default());

        self.binding_manager.add_resource(id, view);
        Texture {
            gpu: self.clone(),
            id,
            texture,
            size: Vector3::new(size.x, size.y, 1),
            _format: PhantomData,
        }
    }
}

impl<Format: TextureFormat> Bindable for Texture<Format> {
    fn resource_id(&self) -> BindableResourceId {
        BindableResourceId::Texture(self.id)
    }

    fn binding_type(&self) -> BindingType {
        BindingType::Texture {
            sample_type: TextureSampleType::Float { filterable: true },
            view_dimension: if self.size.z > 1 {
                TextureViewDimension::D3
            } else {
                TextureViewDimension::D2
            },
            multisampled: false,
        }
    }
}

// impl Drop for Texture {
//     fn drop(&mut self) {
//         self.gpu.binding_manager.remove_resource(self.id);
//     }
// }
