use std::sync::atomic::{AtomicU64, Ordering};

macro_rules! id_type {
    ($name:ident) => {
        #[derive(PartialEq, Eq, Hash, Copy, Clone)]
        pub struct $name(u64);

        impl $name {
            pub(crate) fn new() -> Self {
                static ID: AtomicU64 = AtomicU64::new(0);
                Self(ID.fetch_add(1, Ordering::Relaxed))
            }
        }
    };
}

id_type!(BufferId);
id_type!(PipelineId);
id_type!(AccelerationStructureId);
