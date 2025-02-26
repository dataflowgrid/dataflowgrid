/* This file is part of dataFlowGrid. See file LICENSE for full license details. (c) 2025 Alexander Zich */

#![allow(dead_code)]

#[derive(PartialEq, Debug)]
pub enum OrderedMultiDictEntry<'a> {
    String { str: &'a str},
    Decimal { int: usize},
    Null,
    List { list: Vec<OrderedMultiDictEntry<'a>>},
    Dict { dict: OrderedMultiDict<'a>},
    True,
    False
}

#[derive(PartialEq, Debug)]
struct OrderedMultiDict<'a> {
    entries: Vec<OrderedMultiDictEntry<'a>>,
    keys: Vec<&'a str>
}

impl<'a> OrderedMultiDict<'a> {
    fn new() -> OrderedMultiDict<'a> {
        OrderedMultiDict {
            entries: Vec::new(),
            keys: Vec::new()
        }
    }

    fn insert(&mut self, key: &'a str, value: OrderedMultiDictEntry<'a>) {
        self.keys.push(key);
        self.entries.push(value);
    }

    fn get(&self, key: &'a str) -> Option<&OrderedMultiDictEntry> {
        let index = self.keys.iter().position(|x| *x == key);
        match index {
            Some(i) => Some(&self.entries[i]),
            None => None
        }
    }

    fn remove(&mut self, key: &'a str) {
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

    #[test]
    fn empty() {
        let result = OrderedMultiDict::new();
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
