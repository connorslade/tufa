use nalgebra::{Vector2, Vector3};
use wgpu::{
    BindingType, Extent3d, TexelCopyBufferLayout, TextureDescriptor, TextureDimension,
    TextureSampleType, TextureUsages, TextureViewDescriptor, TextureViewDimension,
};

use crate::{gpu::Gpu, misc::ids::TextureId, TEXTURE_FORMAT};

use super::{Bindable, BindableResourceId};

pub struct Texture {
    gpu: Gpu,

    pub(crate) id: TextureId,
    texture: wgpu::Texture,
    size: Vector3<u32>,
}

impl Texture {
    pub fn upload(&self, data: &[u8]) {
        assert_eq!(
            data.len(),
            self.size.iter().copied().product::<u32>() as usize
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
}

impl Gpu {
    pub fn create_texture_2d(&self, size: Vector2<u32>) -> Texture {
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
            format: TEXTURE_FORMAT,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
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
        }
    }
}

impl Bindable for Texture {
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
