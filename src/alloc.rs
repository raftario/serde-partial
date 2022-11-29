extern crate alloc;

use alloc::collections::{
    btree_map::{BTreeMap, Keys},
    BTreeSet,
};
use core::iter::Map;

use serde::Serialize;

use crate::{Field, Partial, SerializeFilter, SerializePartial};

impl<'a, K, V> SerializePartial<'a> for BTreeMap<K, V>
where
    K: Ord + AsRef<str> + Serialize + 'a,
    V: Serialize + 'a,
{
    #[allow(clippy::type_complexity)]
    type Fields = Map<Keys<'a, K, V>, fn(&'a K) -> Field<'a, Self>>;
    type Filter = BTreeSet<Field<'a, Self>>;

    fn with_fields<F, I>(&'a self, select: F) -> Partial<'a, Self>
    where
        F: FnOnce(Self::Fields) -> I,
        I: IntoIterator<Item = Field<'a, Self>>,
    {
        let fields: Self::Fields = self.keys().map(|k| Field::new(k.as_ref()));
        let filter = select(fields).into_iter().collect();
        Partial {
            value: self,
            filter,
        }
    }
}

impl<'a, K, V> SerializeFilter<BTreeMap<K, V>> for BTreeSet<Field<'a, BTreeMap<K, V>>> {
    fn skip(&self, field: Field<'_, BTreeMap<K, V>>) -> bool {
        self.contains(&field)
    }
}
