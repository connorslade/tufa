use std::sync::atomic::{AtomicU64, Ordering};

use crate::bindings::BindableResourceId;

macro_rules! id_types {
    {$($name:ident),*} => {
       $(
            #[derive(PartialEq, Eq, Hash, Copy, Clone)]
            pub struct $name(u64);

            impl $name {
                pub(crate) fn new() -> Self {
                    static ID: AtomicU64 = AtomicU64::new(0);
                    Self(ID.fetch_add(1, Ordering::Relaxed))
                }
            }
       )*
    };
}

macro_rules! into_bindable_resource {
    {$($name:ident => $id_name:ident),*} => {
        $(
            impl From<$name> for BindableResourceId {
                fn from(value: $name) -> BindableResourceId {
                    BindableResourceId::$id_name(value)
                }
            }
        )*
    };
}

id_types! {
    BufferId,
    TextureId,
    PipelineId,
    AccelerationStructureId,

    TextureCollectionId
}

into_bindable_resource! {
    BufferId => Buffer,
    TextureId => Texture,
    AccelerationStructureId => AccelerationStructure
}
