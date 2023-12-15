use std::ops::Add;

/// Iterator adapter that offsets a sequence of T by a constant value
#[derive(Debug, Default, Copy, Clone)]
pub struct Offset<I, T> {
    inner: I,
    delay: T,
}

impl<I, T> Iterator for Offset<I, T>
where
    I: Iterator<Item = T>,
    T: Copy + Add<T, Output = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|t| t + self.delay)
    }
}

pub trait OffsetIterator<T>: Sized {
    fn offset(self, delay: T) -> Offset<Self, T>;
}

impl<I, T> OffsetIterator<T> for I
where
    I: Iterator<Item = T>,
{
    fn offset(self, delay: T) -> Offset<Self, T> {
        Offset { inner: self, delay }
    }
}

