//! Determines if a buffer can be be written to in a shader.
//! This can be the difference between the following wgsl code:
//! ```wgsl
//! @group(0) @binding(0) var<storage, read> data: Data;
//! @group(0) @binding(0) var<storage, read_write> data: Data;
//! ```

pub trait Mutability {}

pub struct Mutable;
pub struct Immutable;

impl Mutability for Mutable {}
impl Mutability for Immutable {}
