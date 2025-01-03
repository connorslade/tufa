pub trait Mutability {}

pub struct Mutable;
pub struct Immutable;

impl Mutability for Mutable {}
impl Mutability for Immutable {}
