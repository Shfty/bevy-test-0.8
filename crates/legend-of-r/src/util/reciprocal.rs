/// Trait for type-level reciprocality
pub trait Reciprocal {
    type Reciprocal: Reciprocal<Reciprocal = Self>;

    fn reciprocal(&self) -> Self::Reciprocal;
}

