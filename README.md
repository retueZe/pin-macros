# `pin-macros`

This library is primarly used to simplify the proccess of working with self-referencial structures.

## Required knowledge

To read this document further, you should have:

- To know following types:
    - `std::pin::Pin`;
    - `std::pin::Unpin`;
    - `std::mem::MaybeUninit`;
    - `std::marker::PhantomPinned`;
- An understanding of the difference between movable and immovable types;
- Knowledge of how to define an immovable type and how to mark a value (variable, parameter, etc.) as immovable.

## Getting Started

```rust
use std::{pin::Pin, marker::PhantomPinned};
use pin_macros::{pin_init, pin_new};

struct SelfReferential<'a> {
    self_ref: Pin<&'a mut SelfReferential<'a>>,
    val: u32,
    marker: PhantomPinned,
}
impl<'a> SelfReferential<'a> {
    // this is a syntaxic sugar for
    // `pub fn init(ptr: Pin<&'a mut MaybeUninit<Self>>, val: u32) -> Pin<&'a mut Self> { ... }`
    pin_init!(pub fn init<'a>(this, val: u32) {
        // macro available only inside `pin_init!` scope
        // the structure is immovable, it is safe to use a mutable self-reference
        // since it's garanteed that we have only one mutable ref to this value
        // and it doesn't go outside the structure's private scope
        this.self_ref = pin_init_clone!();
        this.val = val;
        this.marker = PhantomPinned::default();
    });
}

fn main() {
    // allocates an immovable value on stack and stores
    // a `Pin<&mut SelfReferencial>` in `self_ref`
    pin_new!(mut self_ref: SelfReferential = init(123));
}
```

## Macros summary

In this section, by `Self`, with a lifetime `'a`, we will mean the immovable type we are working with.

### `pin_new!`

This macro allocates an immovable value on the stack, using `MaybeUninit::<Self>::uninit()`, and then initializes it using the `Self::init` method, storing the initialized `Pin<&mut Self>` pointer in a variable. The variable may be mutable or immutable, depending on the passed tokens.

```rust
fn main() {
    pin_new!(val: T = init(...));
    // OR
    pin_new!(mut val: T = init(...));
}
```

### `pin_init!`

This macro defines an initialization method in an `impl`. It consumes the following tokens:

1. An optional `pub`;
2. A method name;
3. A lifetime (should be `'a`);
4. A variable name for the `&'a mut Self` pointer;
5. An optional list of argument definitions;
6. A block in which you are free to write your initialization code.

It is basically syntactic sugar:

```rust
pin_init!(pub fn init<'a>(this, arg1: u32, arg2: i32) {
    this.arg1 = arg1;
    this.arg2 = arg2;
})
// CONVERTED TO
pub fn init(__ptr: Pin<&'a mut MaybeUninit<Self>>, arg1: u32, arg2: i32) -> Pin<&'a mut Self> {
    // defines `pin_init_xxx!` macros

    let this: &'a mut Self = ...;

    this.arg1 = arg1;
    this.arg2 = arg2;

    unsafe { Pin::new_unchecked(this) }
}
```

#### `pin_init_clone!`

This macro returns a pointer to the already initialized value from the future (`Pin<&'a mut T>`). Since the value is immovable, we can know the addresses of the value and all its fields before the initialization code runs. While the results of `pin_init_clone!` calls are owned by `Self` fields, and the fields are not exposed outside of `Self`'s private scope, it is safe to have multiple mutable references inside.

```rust
pin_init!(... {
    this.pointer_to_itself = pin_init_clone!();
})
```

#### `pin_init_field!`

This macro returns a `Pin<&'a mut MaybeUninit<F>>` pointer, where `F` is a field value type of `Self`. This is used when `Self` owns another immovable value, and we need to initialize it.

```rust
struct Outer<'a> {
    inner: Inner<'a>,
    ...
}

impl<'a> Outer<'a> {
    pin_init!(... {
        Inner::init(pin_init_field!(inner: Inner), ...);
    })
}
```

### `pin_field_init!`

This macro is used to initialize `Option<F>` during the `'a` lifetime but outside the `Self::init` call lifetime, where `F` is a field value type of `Self`. It has two forms: one for owned immovable values and another for anything else.

```rust
// initialization of owned immovable value
pub fn init_during_runtime(self: Pin<&'a mut Self>, ...) {
    // under the hood, we fill the option with `Some(MaybeUninit::<F>::uninit().assume_init())`,
    // and initialize the value
    pin_field_init!(Inner: init(self.inner, ...));
}
```
```rust
// initialization of `Option<(&'a mut F1, &'a mut F2)>`
pub fn init_during_runtime(self: Pin<&'a mut Self>) {
    // we obtain mutable refs to `field1` and `field2`, and then accumulate them in the `dest_field`
    pin_field_init!(self: |field1, field2 => dest_field| (&mut field1, &mut field2))
}
```

### `field_pin!` & `field_unpin!`

These macros are used as wrappers for `self.field` calls. Since our `self` is always wrapped in `Pin`, we cannot simply access a field value. The `field_pin!` macro is used to create private methods that obtain `Pin<&mut F>`, while `field_unpin!` is used for `&mut F`, where `F` is a field value type of `Self`. Clearly, `field_pin!` should be used for immovable values, and `field_unpin!` should be used for movable values.
