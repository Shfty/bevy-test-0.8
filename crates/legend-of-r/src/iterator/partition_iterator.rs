/// Split an iterator into two at the provided index
pub trait PartitionIterator: Sized + Iterator {
    fn partition_iter(self, at: usize) -> (std::vec::IntoIter<Self::Item>, Self);
}

impl<T> PartitionIterator for T
where
    T: Iterator,
{
    fn partition_iter(mut self, at: usize) -> (std::vec::IntoIter<Self::Item>, Self) {
        let mut foo = vec![];
        for _ in 0..at {
            if let Some(bar) = self.next() {
                foo.push(bar)
            } else {
                break;
            }
        }

        (foo.into_iter(), self)
    }
}

