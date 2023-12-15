use bevy::prelude::{default, Component};
use rand::prelude::SliceRandom;

#[derive(Debug, Default, Clone, Component)]
pub struct BagRandomizer<T> {
    pub set: Vec<T>,
    pub count: usize,
    _bag: Vec<T>,
}

impl<T> BagRandomizer<T>
where
    T: Clone,
{
    pub fn new(set: impl IntoIterator<Item = T>, repeat: usize) -> Self {
        let set = set.into_iter().collect();
        BagRandomizer {
            set,
            count: repeat,
            _bag: default(),
        }
    }
}

impl<T> Iterator for BagRandomizer<T>
where
    T: Clone,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self._bag.is_empty() {
            let mut rng = rand::thread_rng();
            for _ in 0..self.count {
                self._bag
                    .extend(self.set.choose_multiple(&mut rng, self.set.len()).cloned());
            }
        }

        self._bag.pop()
    }
}
