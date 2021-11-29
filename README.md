# generational_token_list
[![](https://docs.rs/generational_token_list/badge.svg)](https://docs.rs/generational_token_list/)
![crates.io](https://img.shields.io/crates/v/generational_token_list.svg)
[![Rust](https://github.com/Agilent/generational_token_list/actions/workflows/rust.yml/badge.svg)](https://github.com/Agilent/generational_token_list/actions/workflows/rust.yml)
![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)

A doubly-linked list backed by [generational-arena](https://github.com/fitzgen/generational-arena). Inspired by [indexlist](https://github.com/steveklabnik/indexlist).

Instead of returning pointers or numerical indices to items this data structure returns opaque `ItemToken`s. 

```rust
fn main() {
    let mut list = GenerationalTokenList::<i32>::new();
    let item1 = list.push_back(10);
    let item2 = list.push_back(20);
    let item3 = list.push_back(30);

    let data = list.into_iter().collect::<Vec<_>>();
    assert_eq!(data, vec![10, 20, 30]);
}
```

Tokens remain valid regardless of other items being inserted or removed. Removing an item invalidates its token. Clearing the list invalidates all tokens.

More details and examples are available in the documentation for the methods.  

## Useful features
There are a couple features that I think make this crate stand out compared to other similar crates. Some crates implement a few of these features but I haven't found one that implements all of them (or at the very least, both 1 and 2).
1. Insertion of items relative to other items

```rust
fn main() {
    let mut list = GenerationalTokenList::<i32>::new();
    let item1 = list.push_back(10);
    list.push_back(20);
    list.insert_after(item1, 300);
    // list: [10, 300, 20]
}
```

2. All push/insert methods have a variant that takes a `FnOnce` allowing creation of items that know their own token
```rust
fn main() {
    struct Meta {
        data: u8,
        my_token: ItemToken,
    }

    let mut list = GenerationalTokenList::new();
    let item1 = list.push_back_with(|token| Meta { data: 1, my_token: token });
    let item2 = list.push_back_with(|token| Meta { data: 2, my_token: token });

    let item1_data = list.head().unwrap();
    assert_eq!(item1, item1_data.my_token);
}
```

3. Passthrough of `get2_mut` method from [generational-arena](https://github.com/fitzgen/generational-arena).
4. Implements `Iter` and `IterMut` traits.

## Safety

The only usage of `unsafe` is in the implementation of `iter_mut`. I don't think there is any other way. 

## Similar crates
- [indexlist](https://github.com/steveklabnik/indexlist)
- [chainlink](https://docs.rs/crate/chainlink/0.1.0)
- [array_linked_list](https://docs.rs/array-linked-list/0.1.0/array_linked_list/index.html)
- [index_list](https://crates.io/crates/index_list)

## TODO
Pull requests are welcome :)

- Implement `Index` and `IndexMut` traits
- Implement `Drain`
- Implement `try_push_*` and `try_insert_*` methods
- Implement flavors of `push_*_with` and `insert_*_with` that allow fallible insertion of items? E.g.:
```rust
pub fn push_back_fallible(&mut self, create: impl FnOnce(ItemToken) -> Result<T>) -> Result<ItemToken> {
    //...
}
```
- Add `no-std` support?'
- Consider adding `#[inline]` to some methods?

## Disclaimer
This is not an official Agilent product. No support is implied.
