#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]

use core::{fmt, marker::PhantomData};

use serde::{ser::SerializeStruct, Serialize, Serializer};

/// Derive macro for the [`SerializePartial`] trait.
pub use serde_partial_macro::SerializePartial;

/// Trait implemented by types which can be partially serialized.
pub trait SerializePartial: Serialize {
    /// Type which provides the list of serializable fields.
    ///
    /// When using the derive macro, this type is a struct with the same fields as the original struct.
    /// It will implement [`IntoIterator`] to make it possible to iterate over the available fields, and [`Copy`] and [`Clone`] for convenience.
    /// It will also have a `FIELDS: Self` associated constant.
    type Fields;
    /// Type which can be used to check whether a serializable field should be skipped.
    type Filter: SerializeFilter<Self>;

    /// Returns a value which forwards the [`Serialize`] implementation but only serializes the selected fields.
    ///
    /// The `select` closure receives an instance of [`Fields`][SerializePartial::Fields] which can than be used to select which fields should be serialized.
    /// The closure can return any type which implements [`IntoIterator`]. This could be an array, but could also be a `Vec` or an [`Iterator`] with fields selected at runtime.
    ///
    /// ## Example
    ///
    /// ```
    /// use serde::Serialize;
    /// use serde_partial::SerializePartial;
    ///
    /// #[derive(Serialize, SerializePartial)]
    /// #[serde(rename_all = "camelCase")]
    /// struct User {
    ///     full_name: &'static str,
    ///     age: u8,
    ///     #[serde(rename = "contact")]
    ///     email: &'static str,
    /// }
    ///
    /// const USER: User = User {
    ///     full_name: "John Doe",
    ///     age: 42,
    ///     email: "john.doe@example.com",
    /// };
    ///
    /// // serialize only the `full_name` field
    /// let filtered = USER.with_fields(|u| [u.full_name]);
    /// let json = serde_json::to_value(&filtered).unwrap();
    /// assert_eq!(
    ///     json,
    ///     serde_json::json!({
    ///         "fullName": USER.full_name,
    ///     })
    /// );
    ///
    /// // the field list doesn't have to be an array
    /// // serialize every field with a name longer than 4 characters
    /// let filtered = USER.with_fields(|u| u.into_iter().filter(|f| f.name().len() > 4));
    /// let json = serde_json::to_value(&filtered).unwrap();
    /// assert_eq!(
    ///     json,
    ///     serde_json::json!({
    ///         "fullName": USER.full_name,
    ///         "contact": USER.email,
    ///     })
    /// );
    ///
    /// // field names respect serde attributes
    /// assert_eq!(<User as SerializePartial>::Fields::FIELDS.full_name.name(), "fullName");
    /// assert_eq!(<User as SerializePartial>::Fields::FIELDS.age.name(), "age");
    /// assert_eq!(<User as SerializePartial>::Fields::FIELDS.email.name(), "contact");
    /// ```
    fn with_fields<F, I>(&self, select: F) -> Partial<'_, Self>
    where
        F: FnOnce(Self::Fields) -> I,
        I: IntoIterator<Item = Field<Self>>;

    /// Same as [`with_fields`][SerializePartial::with_fields] but fields are opt-out instead of opt-in.
    ///
    /// ## Example
    ///
    /// ```
    /// # use serde::Serialize;
    /// # use serde_partial::SerializePartial;
    /// #
    /// # #[derive(Serialize, SerializePartial)]
    /// # #[serde(rename_all = "camelCase")]
    /// # struct User {
    /// #     full_name: &'static str,
    /// #     age: u8,
    /// #     #[serde(rename = "contact")]
    /// #     email: &'static str,
    /// # }
    /// #
    /// # const USER: User = User {
    /// #     full_name: "John Doe",
    /// #     age: 42,
    /// #     email: "john.doe@example.com",
    /// # };
    /// #
    /// let filtered = USER.without_fields(|u| [u.email]);
    /// let json = serde_json::to_value(&filtered).unwrap();
    /// assert_eq!(
    ///     json,
    ///     serde_json::json!({
    ///         "fullName": USER.full_name,
    ///         "age": USER.age,
    ///     })
    /// );
    /// ```
    fn without_fields<F, I>(&self, select: F) -> Partial<'_, Self, InverseFilter<Self>>
    where
        F: FnOnce(Self::Fields) -> I,
        I: IntoIterator<Item = Field<Self>>,
    {
        let Partial { value, filter } = self.with_fields(select);
        Partial {
            value,
            filter: InverseFilter::new(filter),
        }
    }
}

/// Trait implemented by types which can be used to filter the serializable fields of another type.
pub trait SerializeFilter<T: ?Sized> {
    /// Returns whether the specified field should be skipped.
    fn skip(&self, field: Field<T>) -> bool;
}

/// A type which implements [`Serialize`] by forwarding the implementation to the value it references while skipping fields according to its filter.
#[derive(Debug)]
pub struct Partial<'a, T, F = <T as SerializePartial>::Filter>
where
    T: ?Sized + SerializePartial,
{
    /// The value to serialize.
    pub value: &'a T,
    /// The field filter to use.
    pub filter: F,
}
impl<'a, T, F> Clone for Partial<'a, T, F>
where
    T: ?Sized + SerializePartial,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            value: self.value,
            filter: self.filter.clone(),
        }
    }
}
impl<'a, T, F> Copy for Partial<'a, T, F>
where
    T: ?Sized + SerializePartial,
    F: Copy,
{
}

/// Newtype around a field name for the specified type.
#[derive(PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Field<T: ?Sized> {
    name: &'static str,
    _ty: PhantomData<T>,
}
impl<T: ?Sized> Clone for Field<T> {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            _ty: PhantomData,
        }
    }
}
impl<T: ?Sized> Copy for Field<T> {}

/// A [`SerializeFilter`] which inverts the behavior of the filter it wraps.
pub struct InverseFilter<T, F = <T as SerializePartial>::Filter>
where
    T: ?Sized + SerializePartial,
{
    filter: F,
    _ty: PhantomData<T>,
}
impl<T, F> Clone for InverseFilter<T, F>
where
    T: ?Sized + SerializePartial,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            filter: self.filter.clone(),
            _ty: PhantomData,
        }
    }
}
impl<T, F> Copy for InverseFilter<T, F>
where
    T: ?Sized + SerializePartial,
    F: Copy,
{
}

impl<T: ?Sized> Field<T> {
    /// Creates a new field.
    ///
    /// The name should be the serde field name and not the Rust field name.
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            _ty: PhantomData,
        }
    }

    /// Returns the field name.
    pub const fn name(&self) -> &'static str {
        self.name
    }
}

impl<T: ?Sized> fmt::Debug for Field<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Field")
            .field("name", &self.name)
            .field("container", &core::any::type_name::<T>())
            .finish()
    }
}

impl<T, F> InverseFilter<T, F>
where
    T: ?Sized + SerializePartial,
    F: SerializeFilter<T>,
{
    /// Creates a filter which inverts the behavior of the provided one.
    pub const fn new(filter: F) -> Self {
        Self {
            filter,
            _ty: PhantomData,
        }
    }
}

impl<T, F> fmt::Debug for InverseFilter<T, F>
where
    T: ?Sized + SerializePartial,
    F: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("InverseFilter").field(&self.filter).finish()
    }
}

impl<T, F> SerializeFilter<T> for InverseFilter<T, F>
where
    T: ?Sized + SerializePartial,
    F: SerializeFilter<T>,
{
    fn skip(&self, field: Field<T>) -> bool {
        !self.filter.skip(field)
    }
}

impl<T, F> Serialize for Partial<'_, T, F>
where
    T: ?Sized + SerializePartial,
    F: SerializeFilter<T>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let serializer = PartialSerializer {
            s: serializer,
            filter: &self.filter,
            _ty: PhantomData,
        };
        self.value.serialize(serializer)
    }
}

struct PartialSerializer<'a, S, T, F>
where
    S: Serializer,
    T: ?Sized + SerializePartial,
    F: SerializeFilter<T>,
{
    s: S,
    filter: &'a F,
    _ty: PhantomData<T>,
}

struct PartialSerializeStruct<'a, S, T, F>
where
    S: Serializer,
    T: ?Sized + SerializePartial,
    F: SerializeFilter<T>,
{
    ss: S::SerializeStruct,
    filter: &'a F,
    _ty: PhantomData<T>,
}

impl<'a, S, T, F> Serializer for PartialSerializer<'a, S, T, F>
where
    S: Serializer,
    T: ?Sized + SerializePartial,
    F: SerializeFilter<T>,
{
    type Ok = S::Ok;
    type Error = S::Error;

    type SerializeStruct = PartialSerializeStruct<'a, S, T, F>;

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        let PartialSerializer { s, filter, _ty } = self;
        let ss = s.serialize_struct(name, len)?;
        Ok(PartialSerializeStruct { ss, filter, _ty })
    }

    type SerializeSeq = S::SerializeSeq;
    type SerializeTuple = S::SerializeTuple;
    type SerializeTupleStruct = S::SerializeTupleStruct;
    type SerializeTupleVariant = S::SerializeTupleVariant;
    type SerializeMap = S::SerializeMap;
    type SerializeStructVariant = S::SerializeStructVariant;

    fn is_human_readable(&self) -> bool {
        self.s.is_human_readable()
    }
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.s.serialize_bool(v)
    }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.s.serialize_i8(v)
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.s.serialize_i16(v)
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.s.serialize_i32(v)
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.s.serialize_i64(v)
    }
    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        self.s.serialize_i128(v)
    }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.s.serialize_u8(v)
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.s.serialize_u16(v)
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.s.serialize_u32(v)
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.s.serialize_u64(v)
    }
    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        self.s.serialize_u128(v)
    }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.s.serialize_f32(v)
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.s.serialize_f64(v)
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.s.serialize_char(v)
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.s.serialize_str(v)
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.s.serialize_bytes(v)
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.s.serialize_none()
    }
    fn serialize_some<TT: ?Sized>(self, value: &TT) -> Result<Self::Ok, Self::Error>
    where
        TT: Serialize,
    {
        self.s.serialize_some(value)
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.s.serialize_unit()
    }
    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.s.serialize_unit_struct(name)
    }
    fn serialize_unit_variant(
        self,
        name: &'static str,
        index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.s.serialize_unit_variant(name, index, variant)
    }
    fn serialize_newtype_struct<TT: ?Sized>(
        self,
        name: &'static str,
        value: &TT,
    ) -> Result<Self::Ok, Self::Error>
    where
        TT: Serialize,
    {
        self.s.serialize_newtype_struct(name, value)
    }
    fn serialize_newtype_variant<TT: ?Sized>(
        self,
        name: &'static str,
        index: u32,
        variant: &'static str,
        value: &TT,
    ) -> Result<Self::Ok, Self::Error>
    where
        TT: Serialize,
    {
        self.s
            .serialize_newtype_variant(name, index, variant, value)
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.s.serialize_seq(len)
    }
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.s.serialize_tuple(len)
    }
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.s.serialize_tuple_struct(name, len)
    }
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.s.serialize_tuple_variant(name, index, variant, len)
    }
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.s.serialize_map(len)
    }
    fn serialize_struct_variant(
        self,
        name: &'static str,
        index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.s.serialize_struct_variant(name, index, variant, len)
    }
    fn collect_str<TT: ?Sized>(self, value: &TT) -> Result<Self::Ok, Self::Error>
    where
        TT: fmt::Display,
    {
        self.s.collect_str(value)
    }
    fn collect_seq<I>(self, iter: I) -> Result<Self::Ok, Self::Error>
    where
        I: IntoIterator,
        <I as IntoIterator>::Item: Serialize,
    {
        self.s.collect_seq(iter)
    }
    fn collect_map<K, V, I>(self, iter: I) -> Result<Self::Ok, Self::Error>
    where
        K: Serialize,
        V: Serialize,
        I: IntoIterator<Item = (K, V)>,
    {
        self.s.collect_map(iter)
    }
}

impl<S, T, F> SerializeStruct for PartialSerializeStruct<'_, S, T, F>
where
    S: Serializer,
    T: ?Sized + SerializePartial,
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
