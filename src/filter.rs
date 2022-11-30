//! Field filtering

use core::{fmt, marker::PhantomData};

use crate::{Field, SerializePartial};

/// Trait implemented by types which can be used to filter the serializable fields of another type.
pub trait SerializeFilter<T: ?Sized> {
    /// Returns whether the specified field should be skipped.
    fn skip(&self, field: Field<'_, T>) -> bool;

    /// Returns the number of fields which will be serialized given the total field count.
    fn filtered_len(&self, len: Option<usize>) -> Option<usize>;
}

/// A [`SerializeFilter`] which inverts the behavior of the filter it wraps.
pub struct InverseFilter<'a, T, F = <T as SerializePartial<'a>>::Filter>
where
    T: ?Sized + SerializePartial<'a>,
{
    filter: F,
    _ty: PhantomData<&'a T>,
}

impl<'a, T, F> SerializeFilter<T> for InverseFilter<'a, T, F>
where
    T: ?Sized + SerializePartial<'a>,
    F: SerializeFilter<T>,
{
    fn skip(&self, field: Field<'_, T>) -> bool {
        !self.filter.skip(field)
    }

    fn filtered_len(&self, len: Option<usize>) -> Option<usize> {
        match (len, self.filter.filtered_len(len)) {
            (Some(len), Some(filtered_len)) => Some(len - filtered_len),
            _ => None,
        }
    }
}

impl<'a, T, F> InverseFilter<'a, T, F>
where
    T: ?Sized + SerializePartial<'a>,
    F: SerializeFilter<T>,
{
    /// Creates a filter which inverts the behavior of the provided one.
    pub fn new(filter: F) -> Self {
        Self {
            filter,
            _ty: PhantomData,
        }
    }
}

impl<'a, T, F> Clone for InverseFilter<'a, T, F>
where
    T: ?Sized + SerializePartial<'a>,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            filter: self.filter.clone(),
            _ty: PhantomData,
        }
    }
}
impl<'a, T, F> Copy for InverseFilter<'a, T, F>
where
    T: ?Sized + SerializePartial<'a>,
    F: Copy,
{
}

impl<'a, T, F> fmt::Debug for InverseFilter<'a, T, F>
where
    T: ?Sized + SerializePartial<'a>,
    F: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("InverseFilter").field(&self.filter).finish()
    }
}
