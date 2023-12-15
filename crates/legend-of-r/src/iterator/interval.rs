use std::{iter::Successors, ops::Add};

pub fn interval<T>(step: T) -> Successors<T, impl FnMut(&T) -> Option<T>>
where
    T: 'static + Copy + Add<T, Output = T>,
    for<'r> &'r T: Add<T, Output = T>,
{
    std::iter::successors(Some(step), move |acc| Some(acc + step))
}
