use generational_arena::{Arena, Index};

#[derive(Debug)]
pub struct Item<T> {
    data: T,
    previous: Option<ItemId>,
    next: Option<ItemId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemId {
    index: Index,
}

#[derive(Debug)]
pub struct GenerationalIndexList<T> {
    arena: Arena<Item<T>>,
    head: Option<ItemId>,
    tail: Option<ItemId>,
}

impl<T> Default for GenerationalIndexList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> GenerationalIndexList<T> {
    pub fn new() -> Self {
        GenerationalIndexList {
            arena: Arena::new(),
            head: None,
            tail: None,
        }
    }

    pub fn with_capacity(n: usize) -> Self {
        GenerationalIndexList {
            arena: Arena::with_capacity(n),
            head: None,
            tail: None,
        }
    }

    pub fn clear(&mut self) {
        self.arena.clear();
        self.head = None;
        self.tail = None;
    }

    pub fn get(&self, id: ItemId) -> Option<&T> {
        self.arena.get(id.index).map(|i| &i.data)
    }

    pub fn get_mut(&mut self, id: ItemId) -> Option<&mut T> {
        self.arena.get_mut(id.index).map(|i| &mut i.data)
    }

    pub fn get2_mut(&mut self, id1: ItemId, id2: ItemId) -> (Option<&mut T>, Option<&mut T>) {
        let (item1, item2) = self.arena.get2_mut(id1.index, id2.index);
        (item1.map(|i| &mut i.data), item2.map(|i| &mut i.data))
    }

    pub fn is_empty(&self) -> bool {
        self.arena.is_empty()
    }

    fn push_only_item(&mut self, data: T) -> ItemId {
        self.push_only_item_with(|_| data)
    }

    fn push_only_item_with(&mut self, create: impl FnOnce(ItemId) -> T) -> ItemId {
        assert!(self.is_empty());
        let id = self.new_node_with(|id| Item {
            data: create(id),
            previous: None,
            next: None,
        });
        self.head = Some(id);
        self.tail = Some(id);
        return id;
    }

    fn new_node(&mut self, node: Item<T>) -> ItemId {
        self.new_node_with(|_| node)
    }

    fn new_node_with(&mut self, create: impl FnOnce(ItemId) -> Item<T>) -> ItemId {
        let index = self.arena.insert_with(|index| create(ItemId { index }));
        ItemId { index }
    }

    pub fn push_back_with(&mut self, create: impl FnOnce(ItemId) -> T) -> ItemId {
        if self.head.is_none() {
            return self.push_only_item_with(create);
        }

        let old_tail = self.tail.unwrap();

        let ret = self.new_node_with(|id| Item {
            data: create(id),
            previous: Some(old_tail),
            next: None,
        });

        // Fixup old tail
        self.arena.get_mut(old_tail.index).unwrap().next = Some(ret);

        self.tail = Some(ret);
        ret
    }

    pub fn push_back(&mut self, data: T) -> ItemId {
        self.push_back_with(|_| data)
    }

    pub fn push_front_with(&mut self, create: impl FnOnce(ItemId) -> T) -> ItemId {
        if self.head.is_none() {
            return self.push_only_item_with(create);
        }

        let old_head = self.head.unwrap();
        let ret = self.new_node_with(|id| Item {
            data: create(id),
            previous: None,
            next: Some(old_head),
        });

        // Fixup old head
        self.arena.get_mut(old_head.index).unwrap().previous = Some(ret);

        self.head = Some(ret);
        ret
    }

    pub fn push_front(&mut self, data: T) -> ItemId {
        self.push_front_with(|_| data)
    }

    pub fn insert_after_with(&mut self, after: ItemId, create: impl FnOnce(ItemId) -> T) -> ItemId {
        assert!(!self.is_empty());

        let item_id_following_after = self.arena.get(after.index).unwrap().next;
        match item_id_following_after {
            // `after` is in tail position
            None => self.push_back_with(create),
            Some(item_id_following_after) => {
                // `after` is not in tail position, which means we are inserting between two items
                let ret = self.new_node_with(|id| Item {
                    data: create(id),
                    previous: Some(after),
                    next: Some(item_id_following_after),
                });

                let (after_item, item_following_after) = self
                    .arena
                    .get2_mut(after.index, item_id_following_after.index);

                after_item.unwrap().next = Some(ret);
                item_following_after.unwrap().previous = Some(ret);

                ret
            }
        }
    }

    pub fn insert_after(&mut self, after: ItemId, data: T) -> ItemId {
        self.insert_after_with(after, |_| data)
    }

    pub fn insert_before_with(
        &mut self,
        before: ItemId,
        create: impl FnOnce(ItemId) -> T,
    ) -> ItemId {
        assert!(!self.is_empty());

        let item_id_preceding_before = self.arena.get(before.index).unwrap().previous;
        match item_id_preceding_before {
            // `before` is in head position
            None => self.push_front_with(create),
            Some(item_id_preceding_before) => {
                // `before` is not in head position, which means we are inserting between two items
                let ret = self.new_node_with(|id| Item {
                    data: create(id),
                    previous: Some(item_id_preceding_before),
                    next: Some(before),
                });

                let (before_item, item_preceding_before) = self
                    .arena
                    .get2_mut(before.index, item_id_preceding_before.index);

                before_item.unwrap().previous = Some(ret);
                item_preceding_before.unwrap().next = Some(ret);

                ret
            }
        }
    }

    pub fn insert_before(&mut self, before: ItemId, data: T) -> ItemId {
        self.insert_before_with(before, |_| data)
    }

    fn iter(&self) -> Iter<T> {
        Iter {
            list: &self,
            next_item: self.head,
        }
    }
}

pub struct Iter<'a, T>
where
    T: 'a,
{
    list: &'a GenerationalIndexList<T>,
    next_item: Option<ItemId>,
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
    list: GenerationalIndexList<T>,
    next_item: Option<ItemId>,
}

impl<T> IntoIterator for GenerationalIndexList<T> {
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

#[cfg(test)]
mod tests {
    use crate::GenerationalIndexList;

    #[test]
    fn basic() {
        let mut list = GenerationalIndexList::<i32>::new();
        let item1 = list.push_back(10);
        let item2 = list.push_back(20);
        let item3 = list.push_back(30);

        let data = list.iter().collect::<Vec<_>>();
        assert_eq!(data, vec![&10, &20, &30]);
    }

    #[test]
    fn into_iter() {
        let mut list = GenerationalIndexList::<i32>::new();
        let item1 = list.push_back(10);
        let item2 = list.push_back(20);
        let item3 = list.push_back(30);

        let data = list.into_iter().collect::<Vec<_>>();
        assert_eq!(data, vec![10, 20, 30]);
    }
}
