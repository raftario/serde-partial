extern crate std;

use core::{hash::Hash, iter::Map};
use std::collections::{
    hash_map::{HashMap, Keys},
    HashSet,
};

use serde::Serialize;

use crate::{Field, Partial, SerializeFilter, SerializePartial};

impl<'a, K, V> SerializePartial<'a> for HashMap<K, V>
where
    K: Hash + Eq + AsRef<str> + Serialize + 'a,
    V: Serialize + 'a,
{
    #[allow(clippy::type_complexity)]
    type Fields = Map<Keys<'a, K, V>, fn(&'a K) -> Field<'a, Self>>;
    type Filter = HashSet<Field<'a, Self>>;

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

impl<'a, K, V> SerializeFilter<HashMap<K, V>> for HashSet<Field<'a, HashMap<K, V>>> {
    fn skip(&self, field: Field<'_, HashMap<K, V>>) -> bool {
        self.contains(&field)
    }
}
