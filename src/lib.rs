use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const INITIAL_NBUCKETS: usize = 1;

pub struct HashMap<K, V> 
{
    buckets: Vec<Vec<(K, V)>>,
    items: usize,
}

impl<K, V> HashMap<K, V>{
    pub fn new() -> Self {
        HashMap {
            buckets: Vec::new(),
            items: 0,
        }
    }
}

impl<K, V> HashMap<K, V>
where 
    K: Hash + Eq,
{
    fn bucket(&self, key: &K) ->usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() % self.buckets.len() as u64) as usize
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.buckets.is_empty() || self.items > 3 * self.buckets.len() / 4 {
            self.resize();
        }
        let bucket = self.bucket(&key);
        let bucket = &mut self.buckets[bucket];
        self.items += 1;
        for &mut (ref ekey, ref mut evalue) in bucket.iter_mut() {
            if ekey == &key {
                use std::mem;
                return Some(mem::replace(evalue, value));
            }
        }
        bucket.push((key, value));
        None
    }

    pub fn resize(&mut self) {
        let target_size = match self.buckets.len() {
            0 => INITIAL_NBUCKETS,
            n => 2 * n,
        };
        let mut new_buckets = Vec::with_capacity(target_size);
        new_buckets.extend((0..target_size).map(|_| Vec::new()));
        for (key, value) in self.buckets.iter_mut().flat_map(|bucket| bucket.drain(..)) {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            let bucket = (hasher.finish() % new_buckets.len() as u64) as usize;
            new_buckets[bucket].push((key, value));
        }
        use std::mem;
        mem::replace(&mut self.buckets, new_buckets);
    }

    pub fn get(&self, key: &K) ->Option<&V> 
    where
        K: Hash + Eq,
    {
        let bucket = self.bucket(key);
        self.buckets[bucket].iter().find(|&(ref ekey, _)| ekey == key)
            .map(|&(_, ref v)| v)
    }

    pub fn remove(&mut self, key: &K) -> Option<V>{
        let bucket = self.bucket(&key);
        let bucket = &mut self.buckets[bucket];
        let i = bucket.iter().position(|&(ref ekey, _)| ekey == key)?;
        self.items -= 1;
        Some(bucket.swap_remove(i).1)

    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    pub fn is_empty(&self) ->bool {
        self.items == 0
    }

    pub fn len(&self) -> usize {
        self.items
    }
}

pub struct Iter<'a, K: 'a, V: 'a> {
    map: &'a HashMap<K, V>,
    bucket: usize,
    at: usize,
}

// impl<'a, K, V> Iter<'a, K: 'a, V: 'a> {
//     fn new(&'a HashMap<K, V>) -> Self {

//     }
// }

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.map.buckets.get(self.bucket) {
                Some(bucket) => {
                    match bucket.get(self.at) {
                        Some(&(ref ekey, ref evalue)) => {
                            self.at += 1;
                            break Some((ekey, evalue));
                        },
                        None => {
                            self.bucket += 1;
                            self.at = 0;
                            continue;
                        },
                    }
                },
                None => {
                    break None
                },
            }
        }
    }
}

impl<'a, K, V> IntoIterator for &'a HashMap<K, V> {
    type Item = (&'a K, &'a V);

    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            map: self,
            bucket: 0,
            at: 0,
        }
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert() {
        let mut map = HashMap::new();
        map.insert("t", "64");
        map.insert("t1", "64");
        assert!(map.len() == 2);
        assert_eq!(map.remove(&"t1"), Some("64"));
        assert_eq!(map.len(), 1);
        assert_eq!(map.contains_key(&"t1"), false);
        assert_eq!(map.contains_key(&"t"), true);
    }
}
