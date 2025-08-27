use std::ops::Deref;

pub trait ToInnerVec<T> {
    fn inner_vec(&self) -> Vec<T>;
}

impl<T> ToInnerVec<String> for [T]
where
    T: Deref<Target = str>,
{
    fn inner_vec(&self) -> Vec<String> {
        self.iter().map(|value| value.to_string()).collect()
    }
}

macro_rules! impl_number {
    ($ty: ty) => {
        impl<T> ToInnerVec<$ty> for [T]
        where
            T: Deref<Target = $ty>,
        {
            fn inner_vec(&self) -> Vec<$ty> {
                self.iter().map(|value| **value).collect()
            }
        }
    };
}

impl_number!(i32);
