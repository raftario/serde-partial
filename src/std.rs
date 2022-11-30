use core::{
    hash::{BuildHasher, Hash},
    iter::Map,
};
use std::collections::{
    hash_map::{HashMap, Keys},
    HashSet,
};

use serde::Serialize;

use crate::{Field, Partial, SerializeFilter, SerializePartial};

impl<'a, K, V, S> SerializePartial<'a> for HashMap<K, V, S>
where
    K: Hash + Eq + AsRef<str> + Serialize + 'a,
    V: Serialize + 'a,
    S: BuildHasher + Default + 'a,
{
    #[allow(clippy::type_complexity)]
    type Fields = Map<Keys<'a, K, V>, fn(&'a K) -> Field<'a, Self>>;
    type Filter = HashSet<Field<'a, Self>, S>;

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

impl<'a, K, V, S> SerializeFilter<HashMap<K, V, S>> for HashSet<Field<'a, HashMap<K, V, S>>, S>
where
    S: BuildHasher,
{
    fn skip(&self, field: Field<'_, HashMap<K, V, S>>) -> bool {
        !self.contains(&field)
    }

    fn filtered_len(&self, _len: Option<usize>) -> Option<usize> {
        Some(self.len())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Field, SerializePartial};

    use std::collections::HashMap;

    #[test]
    fn hash_map() {
        let map = HashMap::from([("a", "b"), ("c", "d")]);
        let filtered = map.with_fields(|_| [Field::new("a")]);
        assert_eq!(
            serde_json::to_value(&filtered).unwrap(),
            serde_json::json!({ "a": "b" })
        )
    }
}
