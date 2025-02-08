use std::num::NonZeroU32;

use wgpu::{BindingType, TextureSampleType, TextureViewDimension};

use crate::{
    bindings::{Bindable, BindableResourceId, Texture},
    gpu::Gpu,
    misc::ids::TextureCollectionId,
};

pub struct TextureCollection {
    gpu: Gpu,
    id: TextureCollectionId,
}

impl Gpu {
    pub fn create_texture_collection(&self, textures: &[&Texture]) -> TextureCollection {
        let id = TextureCollectionId::new();

        self.binding_manager
            .add_collection(id, textures.iter().map(|x| x.id).collect());

        TextureCollection {
            gpu: self.clone(),
            id,
        }
    }
}

impl Bindable for TextureCollection {
    fn resource_id(&self) -> BindableResourceId {
        BindableResourceId::TextureCollection(self.id)
    }

    fn binding_type(&self) -> BindingType {
        BindingType::Texture {
            sample_type: TextureSampleType::Float { filterable: true },
            view_dimension: TextureViewDimension::D2,
            multisampled: false,
        }
    }

    fn count(&self) -> Option<NonZeroU32> {
        let count = self.gpu.binding_manager.get_collection(self.id).len();
        Some(NonZeroU32::new(count as u32).unwrap())
    }
}

impl Drop for TextureCollection {
    fn drop(&mut self) {
        self.gpu.binding_manager.renove_collection(self.id);
    }
}
