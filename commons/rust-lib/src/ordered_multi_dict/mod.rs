/* This file is part of dataFlowGrid. See file LICENSE for full license details. (c) 2025 Alexander Zich */

#![allow(dead_code)]

#[derive(PartialEq, Debug)]
pub struct OrderedMultiDict<K, V> {
    entries: Vec<V>,
    keys: Vec<K>
}

pub struct OrderedMultiDictIterator<'a,K,V> {
    dict: &'a OrderedMultiDict<K,V>,
    current: usize
}
impl<'a,K,V> Iterator for OrderedMultiDictIterator<'a, K,V> {
    // We can refer to this type using Self::Item
    type Item = (&'a K,&'a V);

    //TODO: this might suffer if the Dict is changed while the iterator is alive
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

impl<K,V> OrderedMultiDict<K,V> {
    pub fn new() -> OrderedMultiDict<K,V> {
        OrderedMultiDict {
            entries: Vec::new(),
            keys: Vec::new()
        }
    }

    pub fn push(&mut self, key: K, value: V) {
        self.keys.push(key);
        self.entries.push(value);
    }

    pub fn last_entry(&self) -> Option<&V> {
        self.entries.last()
    }

    pub fn replace_last_entry(&mut self, value: V) {
        self.entries.pop();
        self.entries.push(value);
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

    pub fn iter(&self) -> OrderedMultiDictIterator<K,V> {
        OrderedMultiDictIterator { dict: &self, current: 0 }
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
        result.push("key", OrderedMultiDictEntry::String { str: "value"});
        assert_eq!(result.length(), 1);
        assert_eq!(result.get("key").unwrap(), &OrderedMultiDictEntry::String { str: "value"});

        result.remove("key");
        assert_eq!(result.length(), 0);
        assert!(result.get("key").is_none());
    }

    #[test]
    fn insert_empty_list() {
        let mut result = OrderedMultiDict::new();
        result.push("key", OrderedMultiDictEntry::List { list: Vec::new()});
        assert_eq!(result.length(), 1);
        assert_eq!(result.get("key").unwrap(), &OrderedMultiDictEntry::List { list: Vec::new()});
    }
}
