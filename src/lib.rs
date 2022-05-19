// SPDX-License-Identifier: MIT

#![cfg_attr(not(feature = "iter-mut"), forbid(unsafe_code))]
#![cfg_attr(feature = "iter-mut", deny(unsafe_code))]

use generational_arena::{Arena, Index};

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
struct Item<T> {
    data: T,
    previous: Option<ItemToken>,
    next: Option<ItemToken>,
}

/// An opaque reference to an item in the list.
///
/// Tokens remain valid for as long as their corresponding item is still in the list. Tokens can be
/// freely copied/cloned.
///
/// # Examples
/// ```
/// # use generational_token_list::{GenerationalTokenList, ItemToken};
/// let mut list = GenerationalTokenList::<&str>::new();
/// let item1: ItemToken = list.push_back("OK then buddy");
/// assert_eq!(list.get(item1), Some(&"OK then buddy"));
/// ```
///
/// Even if you remove other items and/or insert new items, tokens remain valid.
///
/// ```
/// # use generational_token_list::{GenerationalTokenList, ItemToken};
/// let mut list = GenerationalTokenList::new();
/// let item1 = list.push_back(1);
/// let item2 = list.push_back(2);
///
/// assert_eq!(list.get(item2), Some(&2));
///
/// list.remove(item1);
///
/// assert_eq!(list.get(item2), Some(&2));
/// ```
///
/// Trying to `get` an item via an invalid token will return `None`.
///
/// ```
/// # use generational_token_list::{GenerationalTokenList, ItemToken};
/// let mut list = GenerationalTokenList::new();
/// let item1 = list.push_back(1);
/// list.push_back(2);
/// list.remove(item1);
/// assert_eq!(list.get(item1), None);
/// ```
///
/// Even if you re-insert the same data into the same place, the returned old token is still invalid.
///
/// ```
/// # use generational_token_list::{GenerationalTokenList, ItemToken};
/// let mut list = GenerationalTokenList::new();
/// let item1 = list.push_back(1);
/// list.push_back(2);
/// assert_eq!(list.iter().collect::<Vec<_>>(), vec![&1, &2]);
///
/// list.remove(item1);
/// // Token is now invalid
/// assert_eq!(list.iter().collect::<Vec<_>>(), vec![&2]);
/// assert_eq!(list.get(item1), None);
///
/// let item1_new = list.push_front(1);
/// // The list looks the same as it was before `remove`...
/// assert_eq!(list.iter().collect::<Vec<_>>(), vec![&1, &2]);
///
/// // ...but the old token is still invalid.
/// assert_eq!(list.get(item1), None);
/// // And the new token works fine.
/// assert_eq!(list.get(item1_new), Some(&1));
///
/// // You can confirm that item1 != item1_new
/// assert_ne!(item1, item1_new);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemToken {
    index: Index,
}

/// A doubly linked list, backed by [generational-arena](https://github.com/fitzgen/generational-arena).
///
/// See the crate documentation for more.
#[derive(Debug)]
pub struct GenerationalTokenList<T> {
    arena: Arena<Item<T>>,
    head: Option<ItemToken>,
    tail: Option<ItemToken>,
}

impl<T> Default for GenerationalTokenList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> GenerationalTokenList<T> {
    /// Creates a new `GenerationalTokenList<T>`.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::new();
    /// list.push_back(String::from("Hi, friend!"));
    /// ```
    pub fn new() -> Self {
        GenerationalTokenList {
            arena: Arena::new(),
            head: None,
            tail: None,
        }
    }

    /// Creates a new `GenerationalTokenList<T>` with given capacity.
    pub fn with_capacity(n: usize) -> Self {
        GenerationalTokenList {
            arena: Arena::with_capacity(n),
            head: None,
            tail: None,
        }
    }

    /// Returns a reference to the first item in the list, or `None` if list is empty.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::new();
    /// list.push_back("slice");
    /// list.push_back("and");
    /// list.push_back("dice");
    /// assert_eq!(list.head(), Some(&"slice"));
    /// ```
    pub fn head(&self) -> Option<&T> {
        self.head.map(|token| self.get(token).unwrap())
    }

    /// Returns a mutable reference to the first item in the list, or `None` if list is empty.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::new();
    /// list.push_back("sugar");
    /// list.push_back("and");
    /// list.push_back("spice");
    /// *list.head_mut().unwrap() = "WAT";
    /// assert_eq!(list.head(), Some(&"WAT"));
    /// ```
    pub fn head_mut(&mut self) -> Option<&mut T> {
        self.head.map(|token| self.get_mut(token).unwrap())
    }

    /// Returns a reference to the last item in the list, or `None` if list is empty.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::new();
    /// list.push_back("slice");
    /// list.push_back("and");
    /// list.push_back("dice");
    /// assert_eq!(list.tail(), Some(&"dice"));
    /// ```
    pub fn tail(&self) -> Option<&T> {
        self.tail.map(|token| self.get(token).unwrap())
    }

    /// Returns a mutable reference to the last item in the list, or `None` if list is empty.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::new();
    /// list.push_back("sugar");
    /// list.push_back("and");
    /// list.push_back("spice");
    /// *list.tail_mut().unwrap() = "WAT";
    /// assert_eq!(list.tail(), Some(&"WAT"));
    /// ```
    pub fn tail_mut(&mut self) -> Option<&mut T> {
        self.tail.map(|token| self.get_mut(token).unwrap())
    }

    /// Returns the token corresponding to first item in the list, or `None` if list is empty.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::new();
    /// assert_eq!(list.head_token(), None);
    /// let head = list.push_back(1);
    /// list.push_back(2);
    /// list.push_back(3);
    /// assert_eq!(list.head_token(), Some(head));
    /// ```
    pub fn head_token(&self) -> Option<ItemToken> {
        self.head
    }

    /// Returns the token corresponding to last item in the list, or `None` if list is empty.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::new();
    /// assert_eq!(list.tail_token(), None);
    /// list.push_back(1);
    /// list.push_back(2);
    /// let tail = list.push_back(3);
    /// assert_eq!(list.tail_token(), Some(tail));
    /// ```
    pub fn tail_token(&self) -> Option<ItemToken> {
        self.tail
    }

    /// Remove all items from the arena. Invalidates all tokens.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::new();
    /// list.push_back(String::from("moo"));
    /// let moo_2 = list.push_back(String::from("moo"));
    /// list.push_back(String::from("cow"));
    /// list.clear();
    /// assert_eq!(list.get(moo_2), None);
    /// ```
    pub fn clear(&mut self) {
        self.arena.clear();
        self.head = None;
        self.tail = None;
    }

    /// Remove the item identified by given token from the list and return the item. Invalidates the
    /// token. Returns `None` if token is invalid.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::new();
    /// let one = list.push_back(1);
    /// let two = list.push_back(2);
    /// let zero = list.push_front(0);
    /// assert_eq!(list.iter().collect::<Vec<_>>(), vec![&0, &1, &2]);
    ///
    /// assert_eq!(list.remove(one).unwrap(), 1);
    /// // Token `one` is now invalid
    /// assert_eq!(list.get(one), None);
    ///
    /// assert_eq!(list.iter().collect::<Vec<_>>(), vec![&0, &2]);
    /// ```
    pub fn remove(&mut self, token: ItemToken) -> Option<T> {
        let item = self.arena.remove(token.index)?;

        if self.head == Some(token) && self.tail == Some(token) {
            // This was the only item in the list
            self.head = None;
            self.tail = None;
        } else if self.head == Some(token) {
            // This was the head and there is another item after us, so make it the new head
            let next_token = item.next.unwrap();
            let next = self.arena.get_mut(next_token.index).unwrap();
            next.previous = None;
            self.head = Some(next_token);
        } else if self.tail == Some(token) {
            // This was the head and there is another item before us, so make it the new tail
            let prev_token = item.previous.unwrap();
            let prev = self.arena.get_mut(prev_token.index).unwrap();
            prev.next = None;
            self.tail = Some(prev_token);
        } else {
            // We were somewhere in the middle
            let next_token = item.next.unwrap();
            let prev_token = item.previous.unwrap();

            let (next, prev) = self.arena.get2_mut(next_token.index, prev_token.index);

            next.unwrap().previous = Some(prev_token);
            prev.unwrap().next = Some(next_token);
        }

        Some(item.data)
    }

    /// Remove first (head) item from the list and return it. Any tokens pointing to head are invalidated.
    /// Returns `None` if the list is empty.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::new();
    /// list.push_back(1);
    /// list.push_back(2);
    /// list.push_back(3);
    /// assert_eq!(list.len(), 3);
    /// assert_eq!(list.pop_front(), Some(1));
    /// assert_eq!(list.len(), 2);
    /// ```
    pub fn pop_front(&mut self) -> Option<T> {
        self.head.and_then(|token| self.remove(token))
    }

    /// Remove last (tail) item from the list and return it. Any tokens pointing to tail are invalidated.
    /// Returns `None` if the list is empty.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::new();
    /// list.push_back(1);
    /// list.push_back(2);
    /// list.push_back(3);
    /// assert_eq!(list.len(), 3);
    /// assert_eq!(list.pop_back(), Some(3));
    /// assert_eq!(list.len(), 2);
    /// ```
    pub fn pop_back(&mut self) -> Option<T> {
        self.tail.and_then(|token| self.remove(token))
    }

    /// Returns the number of items in the list.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::new();
    /// assert_eq!(list.len(), 0);
    /// list.push_back(1);
    /// list.push_back(2);
    /// list.push_back(3);
    /// assert_eq!(list.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.arena.len()
    }

    /// Returns the capacity of the list.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::<String>::with_capacity(10);
    /// assert_eq!(list.capacity(), 10);
    /// ```
    pub fn capacity(&self) -> usize {
        self.arena.capacity()
    }

    /// Get a reference to the data pointed to by given token, or `None` if token is invalid.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::new();
    /// let item1 = list.push_back(vec![0, 1, 2]);
    /// let item2 = list.push_back(vec![3, 4, 5]);
    /// let item3 = list.push_back(vec![6, 7, 8]);
    /// assert_eq!(list.get(item2).unwrap(), &vec![3, 4, 5])
    /// ```
    pub fn get(&self, token: ItemToken) -> Option<&T> {
        self.arena.get(token.index).map(|i| &i.data)
    }

    /// Get a mutable reference to the data pointed to by given token, or `None` if token is invalid.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::new();
    /// let item1 = list.push_back(vec![0, 1, 2]);
    /// let item2 = list.push_back(vec![3, 4, 5]);
    /// let item3 = list.push_back(vec![6, 7, 8]);
    ///
    /// let item2_data = list.get_mut(item2).unwrap();
    /// item2_data.push(100);
    /// assert_eq!(list.get(item2).unwrap(), &vec![3, 4, 5, 100]);
    /// ```
    pub fn get_mut(&mut self, token: ItemToken) -> Option<&mut T> {
        self.arena.get_mut(token.index).map(|i| &mut i.data)
    }

    /// Get a pair of mutable (exclusive) references to the items identified by `token1` and `token2`.
    ///
    /// # Panics
    /// Panics if `token1` and `token2` correspond to the same item.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::new();
    /// let item1 = list.push_back(vec![0, 1, 2]);
    /// let item2 = list.push_back(vec![3, 4, 5]);
    /// let item3 = list.push_back(vec![6, 7, 8]);
    ///
    /// let (item2_data, item3_data) = list.get2_mut(item2, item3);
    ///
    /// item2_data.unwrap().clear();
    /// item3_data.unwrap().pop();
    /// assert_eq!(list.get(item2).unwrap(), &vec![]);
    /// assert_eq!(list.get(item3).unwrap(), &vec![6, 7])
    /// ```
    ///
    /// This will panic:
    ///
    /// ```should_panic
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::new();
    /// let item1 = list.push_back(vec![0, 1, 2]);
    /// list.push_back(vec![3, 4, 5]);
    ///
    /// let (_, _) = list.get2_mut(item1, item1);
    /// ```
    ///
    /// Like `get_mut`, None will be returned if the token is invalid.
    ///
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::new();
    /// let item1 = list.push_back(vec![0, 1, 2]);
    /// let item2 = list.push_back(vec![3, 4, 5]);
    /// list.remove(item2);
    ///
    /// let (item1_data, item2_data) = list.get2_mut(item1, item2);
    /// assert_eq!(item1_data.unwrap(), &vec![0, 1, 2]);
    /// assert_eq!(item2_data, None);
    /// ```
    pub fn get2_mut(
        &mut self,
        token1: ItemToken,
        token2: ItemToken,
    ) -> (Option<&mut T>, Option<&mut T>) {
        let (item1, item2) = self.arena.get2_mut(token1.index, token2.index);
        (item1.map(|i| &mut i.data), item2.map(|i| &mut i.data))
    }

    /// Returns whether the list is empty.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let empty_list = GenerationalTokenList::<String>::new();
    /// assert!(empty_list.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.arena.is_empty()
    }

    fn push_only_item_with(&mut self, create: impl FnOnce(ItemToken) -> T) -> ItemToken {
        assert!(self.is_empty());
        let token = self.new_node_with(|token| Item {
            data: create(token),
            previous: None,
            next: None,
        });
        self.head = Some(token);
        self.tail = Some(token);
        token
    }

    fn new_node_with(&mut self, create: impl FnOnce(ItemToken) -> Item<T>) -> ItemToken {
        let index = self.arena.insert_with(|index| create(ItemToken { index }));
        ItemToken { index }
    }

    /// Insert the item returned by `create` at the end of the list. Returns a token which
    /// corresponds to the new item.
    ///
    /// This method allows you to add items that know their own token.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::{GenerationalTokenList, ItemToken};
    /// struct Meta {
    ///     data: u8,
    ///     my_token: ItemToken,
    /// }
    ///
    /// let mut list = GenerationalTokenList::new();
    /// let item1 = list.push_back_with(|token| Meta { data: 1, my_token: token});
    /// let item2 = list.push_back_with(|token| Meta { data: 2, my_token: token});
    ///
    /// let item1_data = list.head().unwrap();
    /// assert_eq!(item1, item1_data.my_token);
    /// ```
    pub fn push_back_with(&mut self, create: impl FnOnce(ItemToken) -> T) -> ItemToken {
        if self.head.is_none() {
            return self.push_only_item_with(create);
        }

        let old_tail = self.tail.unwrap();

        let ret = self.new_node_with(|token| Item {
            data: create(token),
            previous: Some(old_tail),
            next: None,
        });

        // Fixup old tail
        self.arena.get_mut(old_tail.index).unwrap().next = Some(ret);

        self.tail = Some(ret);
        ret
    }

    /// Insert a new item at the end of the list. Returns a token which corresponds to the new item.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::new();
    /// let item1 = list.push_back("ITEM1");
    /// assert_eq!(list.get(item1), Some(&"ITEM1"));
    /// ```
    pub fn push_back(&mut self, data: T) -> ItemToken {
        self.push_back_with(|_| data)
    }

    /// Insert the item returned by `create` at the beginning of the list. Returns a token which
    /// corresponds to the new item.
    ///
    /// This method allows you to add items that know their own token.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::{GenerationalTokenList, ItemToken};
    /// struct Meta {
    ///     data: u8,
    ///     my_token: ItemToken,
    /// }
    ///
    /// let mut list = GenerationalTokenList::new();
    /// let item1 = list.push_front_with(|token| Meta { data: 1, my_token: token});
    /// let item2 = list.push_front_with(|token| Meta { data: 2, my_token: token});
    ///
    /// let item2_data = list.head().unwrap();
    /// assert_eq!(item2, item2_data.my_token);
    /// ```
    pub fn push_front_with(&mut self, create: impl FnOnce(ItemToken) -> T) -> ItemToken {
        if self.head.is_none() {
            return self.push_only_item_with(create);
        }

        let old_head = self.head.unwrap();
        let ret = self.new_node_with(|token| Item {
            data: create(token),
            previous: None,
            next: Some(old_head),
        });

        // Fixup old head
        self.arena.get_mut(old_head.index).unwrap().previous = Some(ret);

        self.head = Some(ret);
        ret
    }

    /// Insert a new item at the beginning of the list. Returns a token which corresponds to the new item.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::new();
    /// list.push_back(":)");
    /// list.push_back(":/");
    /// list.push_front(":|");
    /// assert_eq!(list.head(), Some(&":|"));
    /// ```
    pub fn push_front(&mut self, data: T) -> ItemToken {
        self.push_front_with(|_| data)
    }

    /// Insert the item returned by `create` after the item identified by given token. Returns a token
    /// which corresponds to the new item.
    ///
    /// This method allows you to add items that know their own token.
    ///
    /// # Panics
    /// Panics if `after` is an invalid token.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::{GenerationalTokenList, ItemToken};
    /// struct Meta {
    ///     data: u8,
    ///     my_token: ItemToken,
    /// }
    ///
    /// let mut list = GenerationalTokenList::new();
    /// let head = list.push_back_with(|token| Meta { data: 0, my_token: token });
    /// list.push_back_with(|token| Meta { data: 2, my_token: token });
    ///
    /// list.insert_after_with(head, |token| Meta { data: 1, my_token: token });
    ///
    /// assert_eq!(list.into_iter().map(|m| m.data).collect::<Vec<_>>(), vec![0, 1, 2]);
    /// ```
    pub fn insert_after_with(
        &mut self,
        after: ItemToken,
        create: impl FnOnce(ItemToken) -> T,
    ) -> ItemToken {
        assert!(!self.is_empty());

        let item_token_following_after = self.arena.get(after.index).unwrap().next;
        match item_token_following_after {
            // `after` is in tail position
            None => self.push_back_with(create),
            Some(item_token_following_after) => {
                // `after` is not in tail position, which means we are inserting between two items
                let ret = self.new_node_with(|token| Item {
                    data: create(token),
                    previous: Some(after),
                    next: Some(item_token_following_after),
                });

                let (after_item, item_following_after) = self
                    .arena
                    .get2_mut(after.index, item_token_following_after.index);

                after_item.unwrap().next = Some(ret);
                item_following_after.unwrap().previous = Some(ret);

                ret
            }
        }
    }

    /// Insert a new item after the item identified by given token.
    ///
    /// # Panics
    /// Panics if `after` is an invalid token.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::<i32>::new();
    /// let item1 = list.push_back(10);
    /// list.push_back(20);
    /// list.insert_after(item1, 300);
    /// assert_eq!(list.into_iter().collect::<Vec<_>>(), vec![10, 300, 20])
    /// ```
    pub fn insert_after(&mut self, after: ItemToken, data: T) -> ItemToken {
        self.insert_after_with(after, |_| data)
    }

    /// Insert the item returned by `create` before the item identified by given token. Returns a token
    /// which corresponds to the new item.
    ///
    /// This method allows you to add items that know their own token.
    ///
    /// # Panics
    /// Panics if `before` is an invalid token.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::{GenerationalTokenList, ItemToken};
    /// struct Meta {
    ///     data: u8,
    ///     my_token: ItemToken,
    /// }
    ///
    /// let mut list = GenerationalTokenList::new();
    /// list.push_back_with(|token| Meta { data: 0, my_token: token });
    /// let tail = list.push_back_with(|token| Meta { data: 2, my_token: token });
    ///
    /// list.insert_before_with(tail, |token| Meta { data: 1, my_token: token });
    ///
    /// assert_eq!(list.into_iter().map(|m| m.data).collect::<Vec<_>>(), vec![0, 1, 2]);
    /// ```
    pub fn insert_before_with(
        &mut self,
        before: ItemToken,
        create: impl FnOnce(ItemToken) -> T,
    ) -> ItemToken {
        assert!(!self.is_empty());

        let item_token_preceding_before = self.arena.get(before.index).unwrap().previous;
        match item_token_preceding_before {
            // `before` is in head position
            None => self.push_front_with(create),
            Some(item_token_preceding_before) => {
                // `before` is not in head position, which means we are inserting between two items
                let ret = self.new_node_with(|token| Item {
                    data: create(token),
                    previous: Some(item_token_preceding_before),
                    next: Some(before),
                });

                let (before_item, item_preceding_before) = self
                    .arena
                    .get2_mut(before.index, item_token_preceding_before.index);

                before_item.unwrap().previous = Some(ret);
                item_preceding_before.unwrap().next = Some(ret);

                ret
            }
        }
    }

    /// Insert a new item before the item identified by given token.
    ///
    /// # Panics
    /// Panics if `before` is an invalid token.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::<i32>::new();
    /// list.push_back(20);
    /// list.push_back(100);
    /// let item3 = list.push_back(10);
    /// list.insert_before(item3, 300);
    /// assert_eq!(list.into_iter().collect::<Vec<_>>(), vec![20, 100, 300, 10])
    /// ```
    pub fn insert_before(&mut self, before: ItemToken, data: T) -> ItemToken {
        self.insert_before_with(before, |_| data)
    }

    /// Returns an iterator of references to item data in the list.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::<i32>::new();
    /// list.push_back(5);
    /// list.push_back(6);
    /// list.push_back(7);
    ///
    /// let i = list.iter().enumerate().collect::<Vec<_>>();
    /// assert_eq!(i, vec![(0, &5), (1, &6), (2, &7)]);
    /// ```
    pub fn iter(&self) -> Iter<T> {
        Iter {
            inner: self.iter_with_tokens(),
        }
    }

    /// Returns an iterator of pairs of (item tokens, references to item data) in the list.
    pub fn iter_with_tokens(&self) -> IterWithTokens<T> {
        IterWithTokens {
            list: self,
            next_item: self.head,
        }
    }

    /// Returns an iterator of mutable (exclusive) references to item data in the list.
    ///
    /// # Examples
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::<i32>::new();
    /// list.push_back(5);
    /// list.push_back(6);
    /// list.push_back(7);
    ///
    /// for i in list.iter_mut() {
    ///     *i += 10;
    /// }
    ///
    /// assert_eq!(list.into_iter().collect::<Vec<_>>(), vec![15, 16, 17]);
    /// ```
    #[cfg(feature = "iter-mut")]
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            inner: self.iter_with_tokens_mut(),
        }
    }

    /// Returns an iterator of pairs of (item tokens, mutable (exclusive) references to item data) in the list.
    #[cfg(feature = "iter-mut")]
    pub fn iter_with_tokens_mut(&mut self) -> IterWithTokensMut<T> {
        let head = self.head;
        IterWithTokensMut {
            list: self,
            next_item: head,
        }
    }

    /// Returns the token corresponding to the item that is after that identified by `token`. Returns
    /// `None` if no item comes after it (i.e. it is the tail).
    ///
    /// # Panics
    /// Panics if `token` is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::<i32>::new();
    /// let item1 = list.push_back(5);
    /// let item2 = list.push_back(6);
    /// list.push_back(7);
    ///
    /// assert_eq!(list.next_token(item1), Some(item2));
    /// ```
    pub fn next_token(&self, token: ItemToken) -> Option<ItemToken> {
        self.arena.get(token.index).unwrap().next
    }

    /// Returns the token corresponding to the item that is before that identified by `token`. Returns
    /// `None` if no item comes before it (i.e. it is the head).
    ///
    /// # Panics
    /// Panics if `token` is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::<i32>::new();
    /// list.push_back(5);
    /// let item2 = list.push_back(6);
    /// let item3 = list.push_back(7);
    ///
    /// assert_eq!(list.prev_token(item3), Some(item2));
    /// ```
    pub fn prev_token(&self, token: ItemToken) -> Option<ItemToken> {
        self.arena.get(token.index).unwrap().previous
    }

    /// Returns the token corresponding to the item at position `pos`. Returns
    /// `None` if `pos` is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::<i32>::new();
    /// let item1 = list.push_back(5);
    /// let item2 = list.push_back(6);
    /// let item3 = list.push_back(7);
    ///
    /// assert_eq!(list.token_at(0), Some(item1));
    /// assert_eq!(list.token_at(1), Some(item2));
    /// assert_eq!(list.token_at(2), Some(item3));
    /// assert_eq!(list.token_at(3), None);
    /// assert_eq!(list.token_at(4), None);
    /// ```
    pub fn token_at(&self, pos: usize) -> Option<ItemToken> {
        self.iter_with_tokens().nth(pos).map(|ret| ret.0)
    }

    /// Returns the token corresponding to the item at position `pos` from the back. Returns
    /// `None` if `pos` is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::<i32>::new();
    /// let item1 = list.push_back(5);
    /// let item2 = list.push_back(6);
    /// let item3 = list.push_back(7);
    ///
    /// assert_eq!(list.token_at_back(0), Some(item3));
    /// assert_eq!(list.token_at_back(1), Some(item2));
    /// assert_eq!(list.token_at_back(2), Some(item1));
    /// assert_eq!(list.token_at_back(3), None);
    /// assert_eq!(list.token_at_back(4), None);
    /// ```
    pub fn token_at_back(&self, pos: usize) -> Option<ItemToken> {
        if pos >= self.len() {
            return None;
        }

        // TODO: implement DoubleEndedIterator and use that instead
        self.token_at(self.len() - pos - 1)
    }
}

#[cfg(feature = "iter-mut")]
pub struct IterWithTokensMut<'a, T>
where
    T: 'a,
{
    list: &'a mut GenerationalTokenList<T>,
    next_item: Option<ItemToken>,
}

#[cfg(feature = "iter-mut")]
impl<'a, T> Iterator for IterWithTokensMut<'a, T>
where
    T: 'a,
{
    type Item = (ItemToken, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        let next_item = self.next_item?;

        self.list.arena.get_mut(next_item.index).map(|i| {
            self.next_item = i.next;

            #[cfg_attr(feature = "iter-mut", allow(unsafe_code))]
            let data = unsafe { &mut *(&mut i.data as *mut T) };
            (next_item, data)
        })
    }
}

#[cfg(feature = "iter-mut")]
pub struct IterMut<'a, T>
where
    T: 'a,
{
    inner: IterWithTokensMut<'a, T>,
}

#[cfg(feature = "iter-mut")]
impl<'a, T> Iterator for IterMut<'a, T>
where
    T: 'a,
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|d| d.1)
    }
}

pub struct IterWithTokens<'a, T>
where
    T: 'a,
{
    list: &'a GenerationalTokenList<T>,
    next_item: Option<ItemToken>,
}

impl<'a, T> Iterator for IterWithTokens<'a, T>
where
    T: 'a,
{
    type Item = (ItemToken, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let next_item = self.next_item?;

        self.list.arena.get(next_item.index).map(|i| {
            self.next_item = i.next;
            (next_item, &i.data)
        })
    }
}

pub struct Iter<'a, T>
where
    T: 'a,
{
    inner: IterWithTokens<'a, T>,
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: 'a,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|d| d.1)
    }
}

pub struct IntoIter<T> {
    list: GenerationalTokenList<T>,
    next_item: Option<ItemToken>,
}

impl<T> IntoIterator for GenerationalTokenList<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let next_item = self.head;

        IntoIter {
            list: self,
            next_item,
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let next_item = self.next_item?;

        self.list.arena.remove(next_item.index).map(|item| {
            self.next_item = item.next;
            item.data
        })
    }
}

impl<T> GenerationalTokenList<T>
where
    T: PartialEq,
{
    /// Returns `true` if list contains an item that equals `value`, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::<i32>::new();
    /// list.push_back(5);
    /// list.push_back(6);
    /// list.push_back(7);
    ///
    /// assert!(list.contains(&5));
    /// assert!(! list.contains(&100));
    /// ```
    pub fn contains(&self, value: &T) -> bool {
        self.iter().any(|v| v == value)
    }

    /// Returns the token corresponding to the first item in the list comparing equal to `value`,
    /// or `false` if no such item is found.
    ///
    /// If you require a different search strategy (for example, finding all items that compare equal),
    /// consider using `iter` and the methods available on the [`Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html) trait.
    ///
    /// # Examples
    ///
    /// ```
    /// # use generational_token_list::GenerationalTokenList;
    /// let mut list = GenerationalTokenList::<i32>::new();
    /// list.push_back(5);
    /// list.push_back(6);
    /// let seven = list.push_back(7);
    /// let a_different_seven = list.push_back(7);
    /// // Remember, they are different!
    /// assert_ne!(seven, a_different_seven);
    ///
    /// assert_eq!(list.find_token(&7), Some(seven));
    /// assert_eq!(list.find_token(&0), None);
    /// ```
    pub fn find_token(&self, value: &T) -> Option<ItemToken> {
        self.arena
            .iter()
            .find(|item| &(*item).1.data == value)
            .map(|(index, _)| ItemToken { index })
    }
}

impl<T> std::ops::Index<ItemToken> for GenerationalTokenList<T> {
    type Output = T;

    fn index(&self, token: ItemToken) -> &Self::Output {
        self.get(token).unwrap()
    }
}

impl<T> std::ops::IndexMut<ItemToken> for GenerationalTokenList<T> {
    fn index_mut(&mut self, token: ItemToken) -> &mut Self::Output {
        self.get_mut(token).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{GenerationalTokenList, Item};

    macro_rules! assert_eq_contents {
        ($list:ident, $right:expr) => {
            // do the lazy thing and just clone the data to compare
            let data = $list.iter().map(Clone::clone).collect::<Vec<_>>();
            pretty_assertions::assert_eq!(data.as_slice(), $right);
        };
    }

    #[test]
    fn push_back() {
        let mut list = GenerationalTokenList::new();
        list.push_back(10);
        list.push_back(20);
        list.push_back(30);

        assert_eq_contents!(list, &[10, 20, 30]);
    }

    #[test]
    fn push_back_internals() {
        let mut list = GenerationalTokenList::new();
        let item1 = list.push_back(10);
        let item2 = list.push_back(20);
        let item3 = list.push_back(30);

        assert_eq!(
            list.arena.iter().collect::<Vec<_>>(),
            vec![
                (
                    item1.index,
                    &Item {
                        data: 10,
                        previous: None,
                        next: Some(item2)
                    }
                ),
                (
                    item2.index,
                    &Item {
                        data: 20,
                        previous: Some(item1),
                        next: Some(item3)
                    }
                ),
                (
                    item3.index,
                    &Item {
                        data: 30,
                        previous: Some(item2),
                        next: None
                    }
                )
            ]
        );
    }

    #[test]
    fn remove_invalid() {
        let mut list = GenerationalTokenList::new();
        let item1 = list.push_back(10);
        list.push_back(20);
        assert_eq_contents!(list, &[10, 20]);
        assert_eq!(list.remove(item1), Some(10));
        assert_eq_contents!(list, &[20]);
        assert_eq!(list.remove(item1), None);
        assert_eq_contents!(list, &[20]);
    }

    #[test]
    fn remove_only() {
        let mut list = GenerationalTokenList::new();
        let item1 = list.push_back(10);

        assert_eq!(list.head, Some(item1));
        assert_eq!(list.tail, Some(item1));
        assert_eq!(
            list.arena.iter().collect::<Vec<_>>(),
            vec![(
                item1.index,
                &Item {
                    data: 10,
                    previous: None,
                    next: None
                }
            ),]
        );

        list.remove(item1);

        assert_eq!(list.head, None);
        assert_eq!(list.tail, None);
        assert_eq!(list.arena.into_iter().collect::<Vec<_>>(), vec![]);
    }

    #[test]
    fn remove_head() {
        let mut list = GenerationalTokenList::new();
        let item1 = list.push_back(10);
        let item2 = list.push_back(20);

        assert_eq!(list.head, Some(item1));
        assert_eq!(list.tail, Some(item2));
        assert_eq!(
            list.arena.iter().collect::<Vec<_>>(),
            vec![
                (
                    item1.index,
                    &Item {
                        data: 10,
                        previous: None,
                        next: Some(item2)
                    }
                ),
                (
                    item2.index,
                    &Item {
                        data: 20,
                        previous: Some(item1),
                        next: None,
                    }
                )
            ]
        );

        list.remove(item1);

        assert_eq!(list.head, Some(item2));
        assert_eq!(list.tail, Some(item2));
        assert_eq!(
            list.arena.iter().collect::<Vec<_>>(),
            vec![(
                item2.index,
                &Item {
                    data: 20,
                    previous: None,
                    next: None,
                }
            )]
        );
    }

    #[test]
    fn remove_tail() {
        let mut list = GenerationalTokenList::new();
        let item1 = list.push_back(10);
        let item2 = list.push_back(20);

        assert_eq!(list.head, Some(item1));
        assert_eq!(list.tail, Some(item2));
        assert_eq!(
            list.arena.iter().collect::<Vec<_>>(),
            vec![
                (
                    item1.index,
                    &Item {
                        data: 10,
                        previous: None,
                        next: Some(item2)
                    }
                ),
                (
                    item2.index,
                    &Item {
                        data: 20,
                        previous: Some(item1),
                        next: None,
                    }
                )
            ]
        );

        list.remove(item2);

        assert_eq!(list.head, Some(item1));
        assert_eq!(list.tail, Some(item1));
        assert_eq!(
            list.arena.iter().collect::<Vec<_>>(),
            vec![(
                item1.index,
                &Item {
                    data: 10,
                    previous: None,
                    next: None,
                }
            )]
        );
    }

    #[test]
    fn remove_middle() {
        let mut list = GenerationalTokenList::new();
        let item1 = list.push_back(10);
        let item2 = list.push_back(20);
        let item3 = list.push_back(30);
        let item4 = list.push_back(40);

        assert_eq!(list.head, Some(item1));
        assert_eq!(list.tail, Some(item4));
        assert_eq!(
            list.arena.iter().collect::<Vec<_>>(),
            vec![
                (
                    item1.index,
                    &Item {
                        data: 10,
                        previous: None,
                        next: Some(item2)
                    }
                ),
                (
                    item2.index,
                    &Item {
                        data: 20,
                        previous: Some(item1),
                        next: Some(item3),
                    }
                ),
                (
                    item3.index,
                    &Item {
                        data: 30,
                        previous: Some(item2),
                        next: Some(item4),
                    }
                ),
                (
                    item4.index,
                    &Item {
                        data: 40,
                        previous: Some(item3),
                        next: None,
                    }
                )
            ]
        );

        list.remove(item2);

        assert_eq!(list.head, Some(item1));
        assert_eq!(list.tail, Some(item4));
        assert_eq!(
            list.arena.iter().collect::<Vec<_>>(),
            vec![
                (
                    item1.index,
                    &Item {
                        data: 10,
                        previous: None,
                        next: Some(item3),
                    }
                ),
                (
                    item3.index,
                    &Item {
                        data: 30,
                        previous: Some(item1),
                        next: Some(item4),
                    }
                ),
                (
                    item4.index,
                    &Item {
                        data: 40,
                        previous: Some(item3),
                        next: None,
                    }
                )
            ]
        );

        list.remove(item3);
        assert_eq!(list.head, Some(item1));
        assert_eq!(list.tail, Some(item4));
        assert_eq!(
            list.arena.iter().collect::<Vec<_>>(),
            vec![
                (
                    item1.index,
                    &Item {
                        data: 10,
                        previous: None,
                        next: Some(item4),
                    }
                ),
                (
                    item4.index,
                    &Item {
                        data: 40,
                        previous: Some(item1),
                        next: None,
                    }
                )
            ]
        );
    }

    #[test]
    fn pop_front() {
        let mut list = GenerationalTokenList::new();
        list.push_back(10);
        list.push_back(20);
        list.push_back(30);
        assert_eq_contents!(list, &[10, 20, 30]);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq_contents!(list, &[20, 30]);
        assert_eq!(list.pop_front(), Some(20));
        assert_eq_contents!(list, &[30]);
        assert_eq!(list.pop_front(), Some(30));
        assert_eq_contents!(list, &[]);
    }

    #[test]
    fn into_iter() {
        let mut list = GenerationalTokenList::<i32>::new();
        list.push_back(10);
        list.push_back(20);
        list.push_back(30);

        let data = list.into_iter().collect::<Vec<_>>();
        assert_eq!(data, vec![10, 20, 30]);
    }

    #[test]
    fn index() {
        let mut list = GenerationalTokenList::<i32>::new();
        let item1 = list.push_back(10);
        let item2 = list.push_back(20);
        let item3 = list.push_back(30);
        assert_eq!(list[item1], 10);
        assert_eq!(list[item2], 20);
        assert_eq!(list[item3], 30);
    }

    #[test]
    fn index_mut() {
        let mut list = GenerationalTokenList::<i32>::new();
        let item1 = list.push_back(10);
        let item2 = list.push_back(20);
        let item3 = list.push_back(30);

        list[item1] = 500;
        list[item3] *= 9;

        assert_eq!(list[item1], 500);
        assert_eq!(list[item2], 20);
        assert_eq!(list[item3], 270);
    }

    #[test]
    fn iter_with_tokens() {
        let mut list = GenerationalTokenList::<i32>::new();
        let item1 = list.push_back(10);
        let item2 = list.push_back(20);
        let item3 = list.push_back(30);

        let mut iter = list.iter_with_tokens();
        assert_eq!(iter.next(), Some((item1, &10)));
        assert_eq!(iter.next(), Some((item2, &20)));
        assert_eq!(iter.next(), Some((item3, &30)));
        assert_eq!(iter.next(), None);
    }

    #[cfg(feature = "iter-mut")]
    #[test]
    fn iter_with_tokens_mut() {
        let mut list = GenerationalTokenList::<i32>::new();
        let item1 = list.push_back(10);
        let item2 = list.push_back(20);
        let item3 = list.push_back(30);

        let mut iter = list.iter_with_tokens_mut();
        let i1 = iter.next().unwrap();
        assert_eq!(i1.0, item1);
        *i1.1 *= 2;
        let i2 = iter.next().unwrap();
        let i3 = iter.next().unwrap();
        *i2.1 *= 3;
        *i3.1 *= 4;
        assert_eq!(i2.0, item2);
        assert_eq!(i3.0, item3);

        let data = list.into_iter().collect::<Vec<_>>();
        assert_eq!(data, vec![20, 60, 120]);
    }
}
