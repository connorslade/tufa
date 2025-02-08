use nalgebra::Vector2;
use wgpu::{
    BindingType, Extent3d, TextureDescriptor, TextureDimension, TextureSampleType, TextureUsages,
    TextureViewDescriptor, TextureViewDimension,
};

use crate::{gpu::Gpu, misc::ids::TextureId, TEXTURE_FORMAT};

use super::{Bindable, BindableResourceId};

pub struct Texture {
    gpu: Gpu,

    pub(crate) id: TextureId,
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
            usage: TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let id = TextureId::new();
        let view = texture.create_view(&TextureViewDescriptor::default());

        self.binding_manager.add_resource(id, view);
        Texture {
            gpu: self.clone(),
            id,
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
            view_dimension: TextureViewDimension::D2,
            multisampled: false,
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        self.gpu.binding_manager.remove_resource(self.id);
    }
}
