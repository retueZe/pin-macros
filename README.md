# `pin-macros`

This library is primarly used to simplify the proccess of working with self-referencial structures.

## Required knowledge

To read this document further, you are have:

- To know following types:
    - `std::pin::Pin`;
    - `std::pin::Unpin`;
    - `std::mem::MaybeUninit`;
    - `std::marker::PhantomPinned`;
- To know what's difference between movable and immovable types;
- To know how to define an immovable type and how to mark a value (variable, parameter, etc) as immovable.

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

In this section, by `Self`, having a lifetime `'a`, will be meant an immovable type with are we working with.

### `pin_new!`

This macro allocates a immovable value on stack, using `MaybeUninit::<Self>::uninit()`, and then initializes it using the `Self::init` method, storing the initialized `Pin<&mut Self>` pointer in a variable. The variable may be mutable or immutable, depending on passed tokens.

```rust
fn main() {
    pin_new!(val: T = init(...));
    // OR
    pin_new!(mut val: T = init(...));
}
```

### `pin_init!`

This macro defines an initialization method in an `impl`. It consumes following tokens:

1. An optional `pub`;
2. A method's name;
3. A lifetime (should be `'a`);
4. A variable name for the `&'a mut Self` pointer;
5. An optional list of argument definitions;
6. A block in which you are free to write your initialization code.

It basically a syntaxic sugar:

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

This macro returns a pointer to the already initialized value from the future (`Pin<&'a mut T>`). Since the value is immovable, we are able to know value's and all its fields' addresses before the initialization code ran. While results of `pin_init_clone!` calls are owned by `Self` fields, and the fields are not exposed our of `Self`'s private scope, it is safe to have multiple mutable references inside.

```rust
pin_init!(... {
    this.pointer_to_itself = pin_init_clone!();
})
```

#### `pin_init_field!`

This macro returns a `Pin<&'a mut MaybeUninit<F>>` pointer, where `F` — a field value type of `Self`. This is used when `Self` owns another immovable value, and we need to initialize it.

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

This macro is used to initialize `Option<F>` during the `'a` lifetime but outside the `Self::init` call lifetime, where `F` — a field value type of `Self`. It has multiple 2 forms: for owned immovable values and for anything else.

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

This macro are used as wrappers for `self.field` calls. Since our `self` is always wrapped in `Pin`, we cannot just get a field value. The `field_pin!` macro is used for creating private methods obtaining `Pin<&mut F>`, and `field_unpin!` — for `&mut F`, where `F` — a field value type of `Self`. Obviously, `field_pin!` should be used for immovable values, and `field_unpin!` — for movable.
