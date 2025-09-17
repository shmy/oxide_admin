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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inner_vec() {
        let vec = vec!["1", "2", "3"];
        let inner_vec = vec.inner_vec();
        assert_eq!(
            inner_vec,
            vec!["1".to_string(), "2".to_string(), "3".to_string()]
        );
    }

    #[test]
    fn test_inner_vec_i32() {
        struct I32Wrapper(i32);

        impl Deref for I32Wrapper {
            type Target = i32;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        let vec = vec![I32Wrapper(1), I32Wrapper(2), I32Wrapper(3)];
        let inner_vec = vec.inner_vec();
        assert_eq!(inner_vec, vec![1, 2, 3]);
    }
}
