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
        let node = if let Some(v) = self.hashmap.get(&key) {
            unsafe {
                (*v.as_ptr()).value = Some(value);
            }
            v.as_ptr()
        } else {
            let node = Node {
                key: Some(key.clone()),
                value: Some(value),
                prev: None,
                next: None,
            };
            self.len += 1;
            Box::into_raw(Box::new(node))
        };

        unsafe {
            let node = NonNull::new_unchecked(node);
            (*node.as_ptr()).prev = Some(self.head);

            (*node.as_ptr()).next = (*self.head.as_ptr()).next;
            let head_next = (*self.head.as_ptr()).next.unwrap();

            (*head_next.as_ptr()).prev = Some(node);
            (*self.head.as_ptr()).next = Some(node);

            self.hashmap.insert(key, node);
        }

        if self.len > self.cap {
            unsafe {
                let last_entry = (*self.tail.as_ptr()).prev.unwrap();
                let key = (*last_entry.as_ptr()).key.clone().unwrap();
                let value = (*last_entry.as_ptr()).value.clone().unwrap();
                let last_prev = (*last_entry.as_ptr()).prev.unwrap();
                (*self.tail.as_ptr()).prev = Some(last_prev);
                (*last_prev.as_ptr()).next = Some(self.tail);
                self.hashmap.remove(&key);

                let boxed = Box::from_raw(last_entry.as_ptr());
                _ = boxed;

                return Some((key, value));
            }
        }
        None
    }

    /// removes values from the hashmap
    pub fn remove(&mut self, key: K) -> Option<V> {
        let value = self.hashmap.remove(&key);
        if let Some(value) = value {
            let val = unsafe {
                let prev = (*value.as_ptr()).prev.unwrap();
                let next = (*value.as_ptr()).next.unwrap();

                (*prev.as_ptr()).next = Some(next);
                (*next.as_ptr()).prev = Some(prev);
                let val = (*value.as_ptr()).value.clone();
                let boxed = Box::from_raw(value.as_ptr());
                _ = boxed;
                val
            };
            self.len -= 1;
            return val;
        }
        None
    }

    /// get value associated with the key
    /// # `Arguments`
    /// - `key` -> key of the mapping
    /// # `Returns`
    /// - None if key not exist, otherwise value associated with the key
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

    /// peek if a value associated to the key is present in the cache
    /// does not promote the entry
    /// # `Arguments`
    /// - `key` -> key of the mapping
    /// # `Returns`
    /// - None if key does not exist, otherwise the value associated with the key
    pub fn peek(&mut self, key: K) -> Option<V> {
        let value = self.hashmap.get(&key);
        if let Some(value) = value {
            let value = unsafe { (*value.as_ptr()).value.clone() };

            return value;
        }
        None
    }

    /// get first entry of the LRU cache
    pub fn get_first(&mut self) -> V {
        unsafe {
            let next = (*self.head.as_ptr()).next.unwrap();

            let value = (*next.as_ptr()).value.clone();

            value.unwrap()
        }
    }

    /// get last entry of the LRU cache
    pub fn get_last(&mut self) -> V {
        unsafe {
            let prev = (*self.tail.as_ptr()).prev.unwrap();

            let value = (*prev.as_ptr()).value.clone();

            value.unwrap()
        }
    }

    /// returns the length of the cache
    pub fn len(&mut self) -> usize {
        self.len
    }

    /// returns `true` if the cache is empty
    pub fn is_empty(&mut self) -> bool {
        self.len == 0
    }
}

impl<K: Hash + Eq + Clone, V: Clone> Drop for LRUCache<K, V> {
    fn drop(&mut self) {
        let mut curr = self.head;
        loop {
            unsafe {
                let next = (*curr.as_ptr()).next;
                if next.is_none() {
                    return;
                }
                let next = next.unwrap();
                let boxed_c = Box::from_raw(curr.as_ptr());
                _ = boxed_c;
                curr = next;
            }
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

    #[test]
    fn test_eviction() {
        let mut lru: LRUCache<u64, u64> = LRUCache::new(1);

        let res = lru.add(1, 1);
        assert!(res.is_none());

        let res = lru.add(2, 2);
        assert_eq!(Some((1, 1)), res);

        assert_eq!(lru.get_first(), 2);

        let value = lru.get(1);
        assert_eq!(None, value);

        let value = lru.get(2);
        assert_eq!(Some(2), value);
    }

    #[test]
    fn test_peek() {
        let mut lru: LRUCache<u64, u64> = LRUCache::new(5);

        lru.add(1, 1);
        lru.add(2, 2);

        assert_eq!(lru.get_first(), 2);
        lru.peek(1);
        assert_eq!(lru.get_first(), 2); // did not promote because of peek, so first is still 2

        lru.get(1);
        assert_eq!(lru.get_first(), 1); // promoted because of get, so first is 1
    }

    #[test]
    fn test_remove() {
        let mut lru: LRUCache<u64, u64> = LRUCache::new(5);

        lru.add(1, 1);
        lru.add(2, 2);
        assert_eq!(lru.len(), 2);
        assert_eq!(lru.get_first(), 2);
        lru.remove(1);
        assert_eq!(lru.get_first(), 2);
        assert_eq!(lru.get_last(), 2);
        assert_eq!(lru.len(), 1);

        let value = lru.get(1);
        assert!(value.is_none())
    }

    #[test]
    fn test_update() {
        let mut lru: LRUCache<u64, u64> = LRUCache::new(5);
        lru.add(1, 1);
        lru.add(2, 2);
        lru.add(1, 3);
        assert_eq!(lru.len(), 2);
        assert_eq!(lru.get_first(), 3);
    }
}
