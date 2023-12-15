#[macro_export]
macro_rules! cell {
    ($lhs:expr, $rhs:expr) => {
        ($lhs, $rhs)
    };
}

#[macro_export]
macro_rules! Cell {
    ($lhs:ty, $rhs:ty) => {
        ($lhs, $rhs)
    };
}

#[macro_export]
macro_rules ! Cons {
    ($ty:ty, $($tt:tt)*) => {
        $crate::Cell!($ty, Cons![$($tt)*])
    };
    ($ty:ty) => {
        $crate::Cell!($ty, Cons![])
    };
    () => {
        ()
    };
}

#[macro_export]
macro_rules ! cons {
    ($expr:expr, $($tt:tt)*) => {
        $crate::cell!($expr, cons![$($tt)*])
    };
    ($expr:expr) => {
        $crate::cell!($expr, cons![])
    };
    () => {
        ()
    };
}

