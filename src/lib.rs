use generational_arena::{Arena, Index};

#[derive(Debug)]
struct Item<T> {
    data: T,
    previous: Option<ItemToken>,
    next: Option<ItemToken>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemToken {
    index: Index,
}

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
    pub fn new() -> Self {
        GenerationalTokenList {
            arena: Arena::new(),
            head: None,
            tail: None,
        }
    }

    pub fn with_capacity(n: usize) -> Self {
        GenerationalTokenList {
            arena: Arena::with_capacity(n),
            head: None,
            tail: None,
        }
    }

    pub fn head(&self) -> Option<&T> {
        self.head.map(|token| self.get(token).unwrap())
    }

    pub fn head_mut(&mut self) -> Option<&mut T> {
        self.head.map(|token| self.get_mut(token).unwrap())
    }

    pub fn tail(&self) -> Option<&T> {
        self.tail.map(|token| self.get(token).unwrap())
    }

    pub fn tail_mut(&mut self) -> Option<&mut T> {
        self.tail.map(|token| self.get_mut(token).unwrap())
    }

    pub fn head_token(&self) -> Option<ItemToken> {
        self.head
    }

    pub fn tail_token(&self) -> Option<ItemToken> {
        self.tail
    }

    pub fn clear(&mut self) {
        self.arena.clear();
        self.head = None;
        self.tail = None;
    }

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

    pub fn pop_front(&mut self) -> Option<T> {
        self.head.and_then(|token| self.remove(token))
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.tail.and_then(|token| self.remove(token))
    }

    pub fn len(&self) -> usize {
        self.arena.len()
    }

    pub fn capacity(&self) -> usize {
        self.arena.capacity()
    }

    pub fn get(&self, token: ItemToken) -> Option<&T> {
        self.arena.get(token.index).map(|i| &i.data)
    }

    pub fn get_mut(&mut self, token: ItemToken) -> Option<&mut T> {
        self.arena.get_mut(token.index).map(|i| &mut i.data)
    }

    pub fn get2_mut(
        &mut self,
        token1: ItemToken,
        token2: ItemToken,
    ) -> (Option<&mut T>, Option<&mut T>) {
        let (item1, item2) = self.arena.get2_mut(token1.index, token2.index);
        (item1.map(|i| &mut i.data), item2.map(|i| &mut i.data))
    }

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
        return token;
    }

    fn new_node(&mut self, node: Item<T>) -> ItemToken {
        self.new_node_with(|_| node)
    }

    fn new_node_with(&mut self, create: impl FnOnce(ItemToken) -> Item<T>) -> ItemToken {
        let index = self.arena.insert_with(|index| create(ItemToken { index }));
        ItemToken { index }
    }

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

    pub fn push_back(&mut self, data: T) -> ItemToken {
        self.push_back_with(|_| data)
    }

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

    pub fn push_front(&mut self, data: T) -> ItemToken {
        self.push_front_with(|_| data)
    }

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

    pub fn insert_after(&mut self, after: ItemToken, data: T) -> ItemToken {
        self.insert_after_with(after, |_| data)
    }

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

    pub fn insert_before(&mut self, before: ItemToken, data: T) -> ItemToken {
        self.insert_before_with(before, |_| data)
    }

    fn iter(&self) -> Iter<T> {
        Iter {
            list: self,
            next_item: self.head,
        }
    }

    fn iter_mut(&mut self) -> IterMut<T> {
        let head = self.head;
        IterMut {
            list: self,
            next_item: head,
        }
    }

    pub fn next_token(&self, token: ItemToken) -> Option<ItemToken> {
        // TODO: unwrap OK?
        self.arena.get(token.index).unwrap().next
    }

    pub fn prev_token(&self, token: ItemToken) -> Option<ItemToken> {
        // TODO: unwrap OK?
        self.arena.get(token.index).unwrap().previous
    }
}

pub struct IterMut<'a, T>
where
    T: 'a,
{
    list: &'a mut GenerationalTokenList<T>,
    next_item: Option<ItemToken>,
}

impl<'a, T> Iterator for IterMut<'a, T>
where
    T: 'a,
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let next_item = self.next_item?;

        self.list.arena.get_mut(next_item.index).map(|i| {
            self.next_item = i.next;
            unsafe { &mut *(&mut i.data as *mut T) }
        })
    }
}

pub struct Iter<'a, T>
where
    T: 'a,
{
    list: &'a GenerationalTokenList<T>,
    next_item: Option<ItemToken>,
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: 'a,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let next_item = self.next_item?;

        self.list.arena.get(next_item.index).map(|i| {
            self.next_item = i.next;
            &i.data
        })
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
    pub fn contains(&self, value: &T) -> bool {
        self.iter().any(|v| v == value)
    }

    pub fn find_token(&self, value: &T) -> Option<ItemToken> {
        self.arena
            .iter()
            .find(|item| &(*item).1.data == value)
            .map(|(index, _)| ItemToken { index })
    }
}

#[cfg(test)]
mod tests {
    use crate::GenerationalTokenList;

    #[test]
    fn basic() {
        let mut list = GenerationalTokenList::<i32>::new();
        let item1 = list.push_back(10);
        let item2 = list.push_back(20);
        let item3 = list.push_back(30);

        let data = list.iter().collect::<Vec<_>>();
        assert_eq!(data, vec![&10, &20, &30]);
    }

    #[test]
    fn into_iter() {
        let mut list = GenerationalTokenList::<i32>::new();
        let item1 = list.push_back(10);
        let item2 = list.push_back(20);
        let item3 = list.push_back(30);

        let data = list.into_iter().collect::<Vec<_>>();
        assert_eq!(data, vec![10, 20, 30]);
    }
}
