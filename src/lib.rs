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
        assert!(self.is_empty());
        let id = self.new_node(Item {
            data,
            previous: None,
            next: None,
        });
        self.head = Some(id);
        self.tail = Some(id);
        return id;
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
        let index = self.arena.insert(node);
        ItemId { index }
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
        if self.head.is_none() {
            return self.push_only_item(data);
        }

        let old_tail = self.tail.unwrap();
        let ret = self.new_node(Item {
            data,
            previous: Some(old_tail),
            next: None,
        });

        // Fixup old tail
        self.arena.get_mut(old_tail.index).unwrap().next = Some(ret);

        self.tail = Some(ret);
        ret
    }

    pub fn push_front(&mut self, data: T) -> ItemId {
        if self.head.is_none() {
            return self.push_only_item(data);
        }

        let old_head = self.head.unwrap();
        let ret = self.new_node(Item {
            data,
            previous: None,
            next: Some(old_head),
        });

        // Fixup old head
        self.arena.get_mut(old_head.index).unwrap().previous = Some(ret);

        self.head = Some(ret);
        ret
    }

    fn iter(&self) -> Iter<T> {
        Iter {
            list: &self,
            next_item: self.head,
        }
    }
}

pub struct Iter<'a, T> where T: 'a {
    list: &'a GenerationalIndexList<T>,
    next_item: Option<ItemId>,
}

impl<'a, T> Iterator for Iter<'a, T>
    where T: 'a
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
