//! Texture sampler.

use wgpu::{AddressMode, BindingType, FilterMode, SamplerBindingType, SamplerDescriptor};

use crate::{gpu::Gpu, misc::ids::SamplerId};

use super::{Bindable, BindableResourceId};

pub struct Sampler {
    gpu: Gpu,

    pub(crate) id: SamplerId,
}

impl Gpu {
    pub fn create_sampler(&self, mode: FilterMode) -> Sampler {
        let sampler = self.device.create_sampler(&SamplerDescriptor {
            label: None,
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: mode,
            min_filter: mode,
            mipmap_filter: mode,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        let id = SamplerId::new();
        self.binding_manager.add_resource(id, sampler);

        Sampler {
            gpu: self.clone(),
            id,
        }
    }
}

impl Bindable for Sampler {
    fn resource_id(&self) -> BindableResourceId {
        BindableResourceId::Sampler(self.id)
    }

    fn binding_type(&self) -> BindingType {
        BindingType::Sampler(SamplerBindingType::Filtering)
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        self.gpu.binding_manager.remove_resource(self.id);
    }
}
