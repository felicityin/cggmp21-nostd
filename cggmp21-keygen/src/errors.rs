use alloc::boxed::Box;
use core::convert::Infallible;

use thiserror_no_std::Error;

pub type BoxedError = Box<dyn core::error::Error + Send + Sync>;

macro_rules! impl_from {
    (impl From for $target:ty {
        $($var:ident: $ty:ty => $new:expr),+,
    }) => {$(
        impl From<$ty> for $target {
            fn from($var: $ty) -> Self {
                $new
            }
        }
    )+}
}

pub(crate) use impl_from;
