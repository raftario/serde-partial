use core::marker::PhantomData;

use serde::ser::{Serialize, SerializeStruct, Serializer};

use crate::{Field, SerializeFilter, SerializePartial};

pub(crate) struct PartialSerializeStruct<'a, S, T, F>
where
    S: Serializer,
    T: ?Sized,
{
    pub(crate) ss: S::SerializeStruct,
    pub(crate) filter: &'a F,
    pub(crate) _ty: PhantomData<T>,
}

impl<'a, S, T, F> SerializeStruct for PartialSerializeStruct<'a, S, T, F>
where
    S: Serializer,
    T: ?Sized + for<'p> SerializePartial<'p>,
    F: SerializeFilter<T>,
{
    type Ok = <S::SerializeStruct as SerializeStruct>::Ok;
    type Error = <S::SerializeStruct as SerializeStruct>::Error;

    fn serialize_field<TT: ?Sized>(
        &mut self,
        key: &'static str,
        value: &TT,
    ) -> Result<(), Self::Error>
    where
        TT: Serialize,
    {
        if self.filter.skip(Field::new(key)) {
            self.skip_field(key)
        } else {
            self.ss.serialize_field(key, value)
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.ss.end()
    }

    fn skip_field(&mut self, key: &'static str) -> Result<(), Self::Error> {
        self.ss.skip_field(key)
    }
}
