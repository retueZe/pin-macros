use std::mem::{self, MaybeUninit};

/// Since `&mut MaybeUninit<T>` is writable, we are allowed to perform the
/// following call, which is unsafe:
/// ```
/// mem::take(transmute_maybe_uninit(ptr), MaybeUninit::uninit());
/// ```
pub unsafe fn transmute_maybe_uninit<T>(ptr: &mut T) -> &mut MaybeUninit<T> {
    mem::transmute(ptr)
}

/// Initializes owned immovable value on stack.
#[macro_export]
macro_rules! pin_new {
    ($varn:ident: $vart:ty = $methodn:ident($($arg:expr),* $(,)?)) => {
        let mut __uninit = std::mem::MaybeUninit::<$vart>::uninit();
        let __uninit_ptr = std::pin::pin!(__uninit);
        let $varn = <$vart>::init(__uninit_ptr, $($($arg),*)?);
    };
    (mut $varn:ident: $vart:ty = $methodn:ident($($arg:expr),* $(,)?)) => {
        let mut __uninit = std::mem::MaybeUninit::<$vart>::uninit();
        let __uninit_ptr = std::pin::pin!(__uninit);
        let mut $varn = <$vart>::init(__uninit_ptr, $($($arg),*)?);
    };
}
/// Defines `Self::init` method, a replacement of the `Self::new` method.
#[macro_export]
macro_rules! pin_init {
    ($v:vis fn $name:ident<$a:lifetime>($this:ident $(, $($argn:ident: $argt:ty),+)? $(,)?) $blk:block) => {
        $v fn $name(
            mut __uninit_ptr: std::pin::Pin<&$a mut std::mem::MaybeUninit<Self>>,
            $($($argn: $argt)+)?
        ) -> std::pin::Pin<&$a mut Self> {
            let __init_ptr = unsafe { __uninit_ptr.as_mut().get_unchecked_mut().as_mut_ptr() };

            /// Clones the potential result of this method. Should be used
            /// Only to speculatively obtain pointers lying inside `Self`.
            macro_rules! pin_init_clone {
                () => {
                    unsafe { std::pin::Pin::new_unchecked(&mut *__init_ptr) }
                };
            }
            /// Gets `Pin<&mut MaybeUninit<F>>`, where `F` — owned immovable type.
            macro_rules! pin_init_field {
                ($fieldn:ident: $fieldt:ty) => {
                    unsafe { std::pin::Pin::new_unchecked($crate::transmute_maybe_uninit(&mut (*__init_ptr).$fieldn)) }
                };
            }

            let $this = unsafe { &mut *__init_ptr };
            $blk;
            unsafe { std::pin::Pin::new_unchecked($this) }
        }
    };
}
/// Generic utility for initializing optional fields of an immovable value
/// after value's primary initialization. Rules summaries:
/// 1. Initializes field of owned immovable type;
/// 2. Initializes self-referencing field from an array of already initialized
/// field value references;
/// 3. A special simpliest case for the 2nd rule.
#[macro_export]
macro_rules! pin_field_init {
    ($fieldt:ty: $methodn:ident($this:ident.$fieldn:ident $(, $($arg:expr)+)? $(,)?)) => {
        unsafe {
            let __field_ptr = &mut $this.as_mut().get_unchecked_mut().$fieldn as *mut Option<$fieldt>;
            *__field_ptr = Some(std::mem::MaybeUninit::uninit().assume_init());

            match &mut *__field_ptr {
                Some(__field) => {
                    let __pinned_field = std::pin::Pin::new_unchecked($crate::transmute_maybe_uninit(__field));
                    <$fieldt>::$methodn(__pinned_field, $($($arg)+)?);
                },
                None => unreachable!(),
            }
        }
    };
    ($this:ident: |$($srcfield:ident),+ => $dstfield:ident| $fieldv:expr) => {{
        let __this_ptr = unsafe { $this.as_mut().get_unchecked_mut() as *mut Self };
        $(let $srcfield = unsafe { &mut (*__this_ptr).$srcfield };)+
        let __dst_ptr = unsafe { &mut (*__this_ptr).$dstfield };
        __dst_ptr.replace($fieldv)
    }};
}
/// Defines a `Pin<&mut F>` getter, where `F` — field type. Use on owned
/// immovable values only.
#[macro_export]
macro_rules! field_pin {
    ($name:ident: $type:ty) => {
        fn $name(self: std::pin::Pin<&mut Self>) -> std::pin::Pin<&mut $type> {
            unsafe { self.map_unchecked_mut(|this| &mut this.$name) }
        }
    }
}
/// Defines a `&mut F` getter, where `F` — field type.
#[macro_export]
macro_rules! field_unpin {
    ($name:ident: $type:ty) => {
        fn $name(self: std::pin::Pin<&mut Self>) -> &mut $type {
            unsafe { self.map_unchecked_mut(|this| &mut this.$name).get_mut() }
        }
    };
}
