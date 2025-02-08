use crate::{bindings::Bindable, misc::ids::TextureId};

pub struct TextureCollection {
    textures: Vec<TextureId>,
}

impl TextureCollection {
    pub fn new(textures: &[TextureId]) -> Self {
        Self {
            textures: textures.to_vec(),
        }
    }
}

impl Bindable for TextureCollection {
    fn resource_id(&self) -> crate::bindings::BindableResourceId {
        todo!()
    }

    fn binding_type(&self) -> wgpu::BindingType {
        todo!()
    }
}
