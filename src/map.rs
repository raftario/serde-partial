use core::marker::PhantomData;

use serde::ser::{Error, Impossible, Serialize, SerializeMap, Serializer};

use crate::{Field, SerializeFilter, SerializePartial};

pub(crate) struct PartialSerializeMap<'a, S, T, F>
where
    S: Serializer,
    T: ?Sized,
{
    pub(crate) sm: S::SerializeMap,
    pub(crate) filter: &'a F,
    pub(crate) _ty: PhantomData<T>,
}

struct KeySerializer<'a, T, F, E>
where
    T: ?Sized,
{
    filter: &'a F,
    _ty: PhantomData<(&'a T, E)>,
}

impl<'a, S, T, F> SerializeMap for PartialSerializeMap<'a, S, T, F>
where
    S: Serializer,
    T: ?Sized + for<'p> SerializePartial<'p>,
    F: SerializeFilter<T>,
{
    type Ok = <S::SerializeMap as SerializeMap>::Ok;
    type Error = <S::SerializeMap as SerializeMap>::Error;

    fn serialize_key<K: ?Sized>(&mut self, _key: &K) -> Result<(), Self::Error>
    where
        K: Serialize,
    {
        Err(Self::Error::custom("cannot perform partial serialization of lone keys, use `serialize_entry` instead if possible"))
    }

    fn serialize_value<V: ?Sized>(&mut self, _value: &V) -> Result<(), Self::Error>
    where
        V: Serialize,
    {
        Err(Self::Error::custom("cannot perform partial serialization of lone values, use `serialize_entry` instead if possible"))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.sm.end()
    }

    fn serialize_entry<K: ?Sized, V: ?Sized>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error>
    where
        K: Serialize,
        V: Serialize,
    {
        let skip = key.serialize(KeySerializer::<'_, T, F, Self::Error> {
            filter: self.filter,
            _ty: PhantomData,
        })?;
        if skip {
            Ok(())
        } else {
            self.sm.serialize_entry(key, value)
        }
    }
}

static KEY_ERR: &str = "key should serialize to a string";

impl<'a, T, F, E> Serializer for KeySerializer<'a, T, F, E>
where
    T: ?Sized + for<'p> SerializePartial<'p>,
    F: SerializeFilter<T>,
    E: Error,
{
    type Ok = bool;
    type Error = E;

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(self.filter.skip(Field::new(v)))
    }

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom(KEY_ERR))
    }
    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom(KEY_ERR))
    }
    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom(KEY_ERR))
    }
    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom(KEY_ERR))
    }
    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom(KEY_ERR))
    }
    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom(KEY_ERR))
    }
    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom(KEY_ERR))
    }
    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom(KEY_ERR))
    }
    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom(KEY_ERR))
    }
    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom(KEY_ERR))
    }
    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom(KEY_ERR))
    }
    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom(KEY_ERR))
    }
    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom(KEY_ERR))
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom(KEY_ERR))
    }
    fn serialize_some<TT: ?Sized>(self, _value: &TT) -> Result<Self::Ok, Self::Error>
    where
        TT: Serialize,
    {
        Err(Self::Error::custom(KEY_ERR))
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom(KEY_ERR))
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom(KEY_ERR))
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom(KEY_ERR))
    }
    fn serialize_newtype_struct<TT: ?Sized>(
        self,
        _name: &'static str,
        _value: &TT,
    ) -> Result<Self::Ok, Self::Error>
    where
        TT: Serialize,
    {
        Err(Self::Error::custom(KEY_ERR))
    }
    fn serialize_newtype_variant<TT: ?Sized>(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _value: &TT,
    ) -> Result<Self::Ok, Self::Error>
    where
        TT: Serialize,
    {
        Err(Self::Error::custom(KEY_ERR))
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Self::Error::custom(KEY_ERR))
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Self::Error::custom(KEY_ERR))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Self::Error::custom(KEY_ERR))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Self::Error::custom(KEY_ERR))
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(Self::Error::custom(KEY_ERR))
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(Self::Error::custom(KEY_ERR))
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Self::Error::custom(KEY_ERR))
    }
}
