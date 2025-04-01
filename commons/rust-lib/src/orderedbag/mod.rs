/* This file is part of dataFlowGrid. See file LICENSE for full license details. (c) 2025 Alexander Zich */

#![allow(dead_code)]

#[derive(PartialEq, Debug)]
pub struct OrderedBag<K, V> {
    entries: Vec<V>,
    keys: Vec<K>
}

pub struct OrderedBagIterator<'a,K,V> {
    dict: &'a OrderedBag<K,V>,
    current: usize
}
impl<'a,K,V> Iterator for OrderedBagIterator<'a, K,V> {
    // We can refer to this type using Self::Item
    type Item = (&'a K,&'a V);

    //TODO: this might suffer if the Bag is changed while the iterator is alive
    fn next(&mut self) -> Option<Self::Item> {
        if self.current<self.dict.entries.len() {
            let r = (self.dict.keys.get(self.current).unwrap(),self.dict.entries.get(self.current).unwrap());
            self.current += 1;
            Some(r)
        } else {
            None
        }
    }
}

impl<K,V> OrderedBag<K,V> {
    pub fn new() -> OrderedBag<K,V> {
        OrderedBag {
            entries: Vec::new(),
            keys: Vec::new()
        }
    }

    pub fn push(&mut self, key: K, value: V) {
        self.keys.push(key);
        self.entries.push(value);
    }

    /// Inserts a key at the end of the list, but does not insert a value.
    /// This is useful if the value to that key is not known yet, but the key is.
    /// The user is responsible to keep track of which keys were inserted without values.
    pub fn insert_key_only(&mut self, key: K) {
        self.keys.push(key);
    }

    pub fn insert_value_only(&mut self, value: V) {
        self.entries.push(value);
    }

    pub fn keys_and_values_in_sync(&self) -> bool {
        self.keys.len() == self.entries.len()
    }
    
    pub fn get(&self, key: K ) -> Option<&V> where K: PartialEq {
        let index = self.keys.iter().position(|x| *x == key);
        match index {
            Some(i) => Some(&self.entries[i]),
            None => None
        }
    }

    pub fn remove(&mut self, key: K) where K: PartialEq {
        let index = self.keys.iter().position(|x| *x == key);
        match index {
            Some(i) => {
                self.keys.remove(i);
                self.entries.remove(i);
            },
            None => ()
        }
    }

    pub fn length(&self) -> usize {
        self.entries.len()
    }

    pub fn iter(&self) -> OrderedBagIterator<K,V> {
        OrderedBagIterator { dict: &self, current: 0 }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(PartialEq, Debug)]
    enum OrderedMultiDictEntry<'a> {
        String { str: &'a str},
        Decimal { int: usize},
        Null,
        List { list: Vec<OrderedMultiDictEntry<'a>>},
        Dict { dict: OrderedBag<&'a str, OrderedMultiDictEntry<'a>>},
        True,
        False,
        None //an entry that should not be there and should be ignored
    }
    
    #[test]
    fn empty() {
        let result = OrderedBag::<&str, OrderedMultiDictEntry>::new();
        assert_eq!(result.length(), 0);
    }

    #[test]
    fn insert_remove() {
        let mut result = OrderedBag::new();
        result.push("key", OrderedMultiDictEntry::String { str: "value"});
        assert_eq!(result.length(), 1);
        assert_eq!(result.get("key").unwrap(), &OrderedMultiDictEntry::String { str: "value"});

        result.remove("key");
        assert_eq!(result.length(), 0);
        assert!(result.get("key").is_none());
    }

    #[test]
    fn insert_empty_list() {
        let mut result = OrderedBag::new();
        result.push("key", OrderedMultiDictEntry::List { list: Vec::new()});
        assert_eq!(result.length(), 1);
        assert_eq!(result.get("key").unwrap(), &OrderedMultiDictEntry::List { list: Vec::new()});
    }
}
