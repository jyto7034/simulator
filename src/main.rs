use std::collections::HashMap;

struct LazyHashMap<K, V, F>
where
    K: Eq + std::hash::Hash,
    F: Fn(&K) -> V,
{
    map: HashMap<K, Option<V>>,
    compute_fn: F,
}

impl<K, V, F> LazyHashMap<K, V, F>
where
    K: Eq + std::hash::Hash,
    F: Fn(&K) -> V,
{
    fn new(compute_fn: F) -> Self {
        LazyHashMap {
            map: HashMap::new(),
            compute_fn,
        }
    }

    fn get(&mut self, key: &K) -> &V {
        if !self.map.contains_key(key) {
            let value = (self.compute_fn)(key);
            self.map.insert(key.clone(), Some(value));
        }

        self.map.get(key).unwrap().as_ref().unwrap()
    }
}

fn main() {
    let mut my_lazy_map = LazyHashMap::new(|key| {
        println!("Computing the value for key: {:?}", key);
        key.to_string()
    });

    let key1 = "key1";
    let key2 = "key2";

    let value1 = my_lazy_map.get(&key1);
    let value2 = my_lazy_map.get(&key2);

    println!("Value 1: {}", value1);
    println!("Value 2: {}", value2);

    let value1_again = my_lazy_map.get(&key1);

    println!("Value 1 (again): {}", value1_again);
}
