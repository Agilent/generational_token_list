# generational_token_list

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

## Useful features
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

```

3. Passthrough of `get2_mut` method from [generational-arena](https://github.com/fitzgen/generational-arena).
4. Implements `Iter` and `IterMut` traits.

## Safety

The only usage of `unsafe` is in the implementation of `iter_mut`. I don't think there is any other way. 

## TODO
Pull requests are welcome :)

- Implement `Index` and `IndexMut` traits
- Add `no-std` support?'
- Consider adding `#[inline]` to some methods?

## Disclaimer
This is not an official Agilent product. No support is implied.
