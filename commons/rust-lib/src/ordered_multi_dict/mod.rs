/* This file is part of dataFlowGrid. See file LICENSE for full license details. (c) 2025 Alexander Zich */

#![allow(dead_code)]

#[derive(PartialEq, Debug)]
pub struct OrderedMultiDict<K, V> {
    entries: Vec<V>,
    keys: Vec<K>
}

impl<K,V> OrderedMultiDict<K,V> {
    fn new() -> OrderedMultiDict<K,V> {
        OrderedMultiDict {
            entries: Vec::new(),
            keys: Vec::new()
        }
    }

    fn insert(&mut self, key: K, value: V) {
        self.keys.push(key);
        self.entries.push(value);
    }

    fn get(&self, key: K ) -> Option<&V> where K: PartialEq {
        let index = self.keys.iter().position(|x| *x == key);
        match index {
            Some(i) => Some(&self.entries[i]),
            None => None
        }
    }

    fn remove(&mut self, key: K) where K: PartialEq {
        let index = self.keys.iter().position(|x| *x == key);
        match index {
            Some(i) => {
                self.keys.remove(i);
                self.entries.remove(i);
            },
            None => ()
        }
    }

    fn length(&self) -> usize {
        self.entries.len()
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
        Dict { dict: OrderedMultiDict<&'a str, OrderedMultiDictEntry<'a>>},
        True,
        False,
        None //an entry that should not be there and should be ignored
    }
    
    #[test]
    fn empty() {
        let result = OrderedMultiDict::<&str, OrderedMultiDictEntry>::new();
        assert_eq!(result.length(), 0);
    }

    #[test]
    fn insert_remove() {
        let mut result = OrderedMultiDict::new();
        result.insert("key", OrderedMultiDictEntry::String { str: "value"});
        assert_eq!(result.length(), 1);
        assert_eq!(result.get("key").unwrap(), &OrderedMultiDictEntry::String { str: "value"});

        result.remove("key");
        assert_eq!(result.length(), 0);
        assert!(result.get("key").is_none());
    }

    #[test]
    fn insert_empty_list() {
        let mut result = OrderedMultiDict::new();
        result.insert("key", OrderedMultiDictEntry::List { list: Vec::new()});
        assert_eq!(result.length(), 1);
        assert_eq!(result.get("key").unwrap(), &OrderedMultiDictEntry::List { list: Vec::new()});
    }
}
