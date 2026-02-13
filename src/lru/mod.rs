//! Fast LRU

#![allow(dead_code)]

use std::{collections::HashMap, hash::Hash, ptr::NonNull};

/// Node for the linked list that is used to bookkeep the LRU cache
#[derive(Debug)]
pub struct Node<K: Hash + Eq + Clone, V: Clone> {
    key: Option<K>,
    value: Option<V>,
    prev: Option<NonNull<Node<K, V>>>,
    next: Option<NonNull<Node<K, V>>>,
}

/// LRU cache struct
#[derive(Debug)]
pub struct LRUCache<K: Hash + Eq + Clone, V: Clone> {
    hashmap: HashMap<K, NonNull<Node<K, V>>>,
    cap: usize,
    len: usize,
    head: NonNull<Node<K, V>>,
    tail: NonNull<Node<K, V>>,
}

impl<K: Hash + Eq + Clone, V: Clone> LRUCache<K, V> {
    /// creates new instance of LRU cache with the capacity
    /// # `Arguments`
    /// - `cap`-> capacity
    /// # `Returns`
    /// - Self
    pub fn new(cap: usize) -> Self {
        let default_node = Node {
            key: None,
            value: None,
            prev: None,
            next: None,
        };
        let head = Box::into_raw(Box::new(default_node));
        let default_node = Node {
            key: None,
            value: None,
            prev: None,
            next: None,
        };
        let tail = Box::into_raw(Box::new(default_node));

        let head = unsafe { NonNull::new_unchecked(head) };

        let tail = unsafe { NonNull::new_unchecked(tail) };
        unsafe {
            (*head.as_ptr()).next = Some(tail);
            (*head.as_ptr()).prev = None;
            (*tail.as_ptr()).prev = Some(head);

            (*tail.as_ptr()).next = None;
        }
        Self {
            hashmap: HashMap::new(),
            cap,
            len: 0,
            head,
            tail,
        }
    }

    /// adds new key to the LRU cache
    /// # `params`
    /// `key`: key of the entry
    /// `value`: value of the entry
    ///
    /// # `Returns`
    /// None if no keys were evicted, (key, value) if a key was evicted
    pub fn add(&mut self, key: K, value: V) -> Option<(K, V)> {
        let node = Node {
            key: Some(key.clone()),
            value: Some(value),
            prev: None,
            next: None,
        };

        let node = Box::into_raw(Box::new(node));
        unsafe {
            let node = NonNull::new_unchecked(node);
            (*node.as_ptr()).prev = Some(self.head);

            (*node.as_ptr()).next = (*self.head.as_ptr()).next;
            let head_next = (*self.head.as_ptr()).next.unwrap();

            (*head_next.as_ptr()).prev = Some(node);
            (*self.head.as_ptr()).next = Some(node);

            self.hashmap.insert(key, node);
        }

        self.len += 1;
        if self.len > self.cap {
            todo!("Implement eviction")
        }
        None
    }

    pub fn get(&mut self, key: K) -> Option<V> {
        let value = self.hashmap.get(&key);

        if let Some(value) = value {
            unsafe {
                let prev = (*value.as_ptr()).prev.unwrap();
                let next = (*value.as_ptr()).next.unwrap();

                (*prev.as_ptr()).next = Some(next);
                (*next.as_ptr()).prev = Some(prev);

                (*value.as_ptr()).prev = Some(self.head);
                (*value.as_ptr()).next = (*self.head.as_ptr()).next;
                let head_next = (*self.head.as_ptr()).next.unwrap();

                (*head_next.as_ptr()).prev = Some(*value);
                (*self.head.as_ptr()).next = Some(*value);

                let value = (*value.as_ptr()).value.clone();
                return value;
            }
        }
        None
    }

    pub fn get_first(&mut self) -> V {
        unsafe {
            let next = (*self.head.as_ptr()).next.unwrap();

            let value = (*next.as_ptr()).value.clone();

            value.unwrap()
        }
    }
}

mod tests {
    #![allow(unused_imports)]
    use crate::lru::LRUCache;

    #[test]
    fn make_lru() {
        let mut lru: LRUCache<u64, u64> = LRUCache::new(5);

        lru.add(1, 1);
        lru.add(2, 2);

        assert_eq!(lru.get_first(), 2);
        lru.get(1);
        assert_eq!(lru.get_first(), 1);

        lru.get(2);
        assert_eq!(lru.get_first(), 2);

        lru.add(3, 3);
        assert_eq!(lru.get_first(), 3);
    }
}
