// -*- coding: utf-8 -*-
// ------------------------------------------------------------------------------------------------
// Copyright © 2021, Douglas Creager.
// Licensed under either of Apache License, Version 2.0, or MIT license, at your option.
// Please see the LICENSE-APACHE or LICENSE-MIT files in this distribution for license details.
// ------------------------------------------------------------------------------------------------

//! This crate provides a replacement for the standard [`Option`][] type where you have full
//! control over how the `None` and `Some` variants are represented in memory.
//!
//! Normally, you don't have to think about this.  The standard [`Option`][] is a perfectly normal
//! `enum`, and the compiler takes care of determining the most efficient in-memory representation.
//! In particular, the compiler knows that certain types have [_niches_][] — in-memory bit patterns
//! that do not represent valid values of the type.  If a type has a niche, then the compiler can
//! use that bit pattern to represent the `None` variant.  This works automatically for most of the
//! types you might care about: in particular, for references and the various `NonZero` types in
//! `std::num`.
//!
//! However, sometimes a type has _multiple_ possible niches, and you need control over which one
//! the compiler chooses to use.  Or, you might have defined a type such that the compiler cannot
//! see that it has a niche available to use.  In this case, you can use the `Niche` and
//! `ControlledOption` types from this crate to take full control over how the `None` and `Some`
//! variants are laid out in memory.
//!
//! [`Option`]: https://doc.rust-lang.org/std/option/enum.Option.html
//! [_niches_]: https://rust-lang.github.io/unsafe-code-guidelines/glossary.html#niche

use std::alloc::Layout;

/// A type should implement `Niche` if its memory representation has any bit patterns that do not
/// represent valid values.  If so, one of those can be used to represent the `None` case of an
/// option.
pub trait Niche: Sized {
    /// The type that is used to store values of `Self` inside of a `ControlledOption`.  This might
    /// be `Self` itself, if your niche is a valid instance of the type, but which violates some
    /// runtime constraint.  But if you cannot easily create your niche as an instance of `Self`,
    /// you can use some other type, you can use some other type instead.
    ///
    /// A word of caution: is it this `Output` type that is stored inside of a `ControlledOption`.
    /// If you want `ControlledOption<Self>` to have the same memory layout as `Self` (so that you
    /// can use `#[repr(transparent)]`, for instance), then you must ensure that `Self` and
    /// `Output` have the same layout, as determined by [`std::alloc::Layout::new`][new], and that
    /// every valid bit pattern for `Self` is be a valid bit pattern for `Output` that returns
    /// `true` for `is_some`.
    ///
    /// [new]: https://doc.rust-lang.org/std/alloc/struct.Layout.html#method.new
    type Output;

    /// Returns the niche value for this type that should be used to represent `None` for a
    /// `ControlledOption`.
    fn none() -> Self::Output;

    /// Returns whether value is the niche value for this type.
    fn is_none(value: &Self::Output) -> bool;

    /// Transforms a non-niche value of this type into its `Output` type.  When `Output` is `Self`,
    /// this will be the identity function.
    fn into_some(value: Self) -> Self::Output;

    /// Transforms a non-niche value of this type from its `Output` type.  When `Output` is `Self`,
    /// this will be the identity function.
    fn from_some(value: Self::Output) -> Self;
}

/// An `Option` type where you have control over the in-memory representation of the `None` and
/// `Some` variants.  See the [module-level documentation][parent] for more information.
///
/// [parent]: index.html
#[repr(transparent)]
pub struct ControlledOption<T>
where
    T: Niche,
{
    value: T::Output,
}

impl<T> ControlledOption<T>
where
    T: Niche,
{
    /// Creates a new `None` instance for this option.
    #[inline]
    pub fn none() -> ControlledOption<T> {
        let value = T::none();
        debug_assert!(T::is_none(&value));
        ControlledOption { value }
    }

    /// Creates a new `Some` instance for this option.
    #[inline]
    pub fn some(value: T) -> ControlledOption<T> {
        let value = T::into_some(value);
        debug_assert!(!T::is_none(&value));
        ControlledOption { value }
    }

    /// Returns `true` is the option is a `None` value.
    #[inline]
    pub fn is_none(&self) -> bool {
        T::is_none(&self.value)
    }

    /// Returns `true` is the option is a `Some` value.
    #[inline]
    pub fn is_some(&self) -> bool {
        !T::is_none(&self.value)
    }

    /// Transforms an [`Option`][] into a `ControlledOption`.
    ///
    /// [`Option`]: https://doc.rust-lang.org/std/option/enum.Option.html
    #[inline]
    pub fn from_option(value: Option<T>) -> ControlledOption<T> {
        value.into()
    }

    /// Transforms a `ControlledOption` into an [`Option`][].  This gives you access to all of the
    /// usual assortment of useful methods that you expect from an `Option`.
    ///
    /// [`Option`]: https://doc.rust-lang.org/std/option/enum.Option.html
    #[inline]
    pub fn into_option(self) -> Option<T> {
        self.into()
    }
}

impl<T> Default for ControlledOption<T>
where
    T: Niche,
{
    #[inline]
    fn default() -> ControlledOption<T> {
        ControlledOption::none()
    }
}

impl<T> From<T> for ControlledOption<T>
where
    T: Niche,
{
    #[inline]
    fn from(value: T) -> ControlledOption<T> {
        ControlledOption::some(value)
    }
}

impl<T> From<Option<T>> for ControlledOption<T>
where
    T: Niche,
{
    #[inline]
    fn from(value: Option<T>) -> ControlledOption<T> {
        match value {
            Some(value) => ControlledOption::some(value),
            None => ControlledOption::none(),
        }
    }
}

impl<T> Into<Option<T>> for ControlledOption<T>
where
    T: Niche,
{
    #[inline]
    fn into(self) -> Option<T> {
        if T::is_none(&self.value) {
            None
        } else {
            Some(T::from_some(self.value))
        }
    }
}

// Normally we would #[derive] all of these traits, but the auto-derived implementations all
// require that T implement the trait as well.  In our case, we (usually) need T::Output to
// implement the traits, not T itself.

impl<T> Clone for ControlledOption<T>
where
    T: Niche,
    T::Output: Clone,
{
    fn clone(&self) -> Self {
        ControlledOption {
            value: self.value.clone(),
        }
    }
}

impl<T> std::fmt::Debug for ControlledOption<T>
where
    T: std::fmt::Debug + Niche,
    T::Output: Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.is_none() {
            write!(f, "ControlledOption::None")
        } else {
            f.debug_tuple("ControlledOption::Some")
                .field(&T::from_some(self.value.clone()))
                .finish()
        }
    }
}

impl<T> PartialEq for ControlledOption<T>
where
    T: Niche,
    T::Output: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }

    fn ne(&self, other: &Self) -> bool {
        self.value.ne(&other.value)
    }
}

impl<T> Eq for ControlledOption<T>
where
    T: Niche,
    T::Output: Eq,
{
}

impl<T> PartialOrd for ControlledOption<T>
where
    T: Niche,
    T::Output: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }

    fn lt(&self, other: &Self) -> bool {
        self.value.lt(&other.value)
    }

    fn le(&self, other: &Self) -> bool {
        self.value.le(&other.value)
    }

    fn gt(&self, other: &Self) -> bool {
        self.value.gt(&other.value)
    }

    fn ge(&self, other: &Self) -> bool {
        self.value.ge(&other.value)
    }
}

impl<T> Ord for ControlledOption<T>
where
    T: Niche,
    T::Output: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }

    fn max(self, other: Self) -> Self {
        ControlledOption {
            value: self.value.max(other.value),
        }
    }

    fn min(self, other: Self) -> Self {
        ControlledOption {
            value: self.value.min(other.value),
        }
    }

    fn clamp(self, min: Self, max: Self) -> Self {
        ControlledOption {
            value: self.value.clamp(min.value, max.value),
        }
    }
}

impl<T> std::hash::Hash for ControlledOption<T>
where
    T: Niche,
    T::Output: std::hash::Hash,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.value.hash(state)
    }
}

//-------------------------------------------------------------------------------------------------
// Structs
//
// The ‘controlled-option-macros’ crate provides a derive macro for the ‘Niche’ trait.  The derived
// implementation depends on the following functions to get access to the field that you want to
// use as the struct's niche.

/// Automatically derives a [`Niche`][] implementation for a struct type.
///
/// You must mark one of the fields with a `#[niche]` attribute.  This field's type must already
/// implement [`Niche`][].  The `None` value for the struct will be uninitialized memory, except
/// for the chosen field, which will be filled in with its `None` niche value.  (This requires that
/// the [`Niche`][] implementation for the field's type must have the same layout for its `Self`
/// and `Output` types.)
pub use controlled_option_macros::Niche;

#[doc(hidden)]
pub fn fill_struct_field_with_none<T>(field: *mut T)
where
    T: Niche,
{
    debug_assert!(Layout::new::<T>() == Layout::new::<T::Output>());
    let repr = field as *mut T::Output;
    unsafe { repr.write(T::none()) };
}

#[doc(hidden)]
pub fn struct_field_is_none<T>(field: *const T) -> bool
where
    T: Niche,
{
    debug_assert!(Layout::new::<T>() == Layout::new::<T::Output>());
    let repr = field as *const T::Output;
    T::is_none(unsafe { &*repr })
}

//-------------------------------------------------------------------------------------------------
// References

impl<'a, T> Niche for &'a T {
    type Output = *const T;

    #[inline]
    fn none() -> Self::Output {
        std::ptr::null()
    }

    #[inline]
    fn is_none(value: &Self::Output) -> bool {
        value.is_null()
    }

    #[inline]
    fn into_some(value: Self) -> Self::Output {
        value
    }

    #[inline]
    fn from_some(value: Self::Output) -> Self {
        unsafe { &*value }
    }
}

impl<'a, T> Niche for &'a mut T {
    type Output = *mut T;

    #[inline]
    fn none() -> Self::Output {
        std::ptr::null_mut()
    }

    #[inline]
    fn is_none(value: &Self::Output) -> bool {
        value.is_null()
    }

    #[inline]
    fn into_some(value: Self) -> Self::Output {
        value
    }

    #[inline]
    fn from_some(value: Self::Output) -> Self {
        unsafe { &mut *value }
    }
}

//-------------------------------------------------------------------------------------------------
// Non-zero types

impl Niche for std::num::NonZeroI8 {
    type Output = i8;

    #[inline]
    fn none() -> Self::Output {
        0
    }

    #[inline]
    fn is_none(value: &Self::Output) -> bool {
        *value == 0
    }

    #[inline]
    fn into_some(value: Self) -> Self::Output {
        value.get()
    }

    #[inline]
    fn from_some(value: Self::Output) -> Self {
        unsafe { Self::new_unchecked(value) }
    }
}

impl Niche for std::num::NonZeroI16 {
    type Output = i16;

    #[inline]
    fn none() -> Self::Output {
        0
    }

    #[inline]
    fn is_none(value: &Self::Output) -> bool {
        *value == 0
    }

    #[inline]
    fn into_some(value: Self) -> Self::Output {
        value.get()
    }

    #[inline]
    fn from_some(value: Self::Output) -> Self {
        unsafe { Self::new_unchecked(value) }
    }
}

impl Niche for std::num::NonZeroI32 {
    type Output = i32;

    #[inline]
    fn none() -> Self::Output {
        0
    }

    #[inline]
    fn is_none(value: &Self::Output) -> bool {
        *value == 0
    }

    #[inline]
    fn into_some(value: Self) -> Self::Output {
        value.get()
    }

    #[inline]
    fn from_some(value: Self::Output) -> Self {
        unsafe { Self::new_unchecked(value) }
    }
}

impl Niche for std::num::NonZeroI64 {
    type Output = i64;

    #[inline]
    fn none() -> Self::Output {
        0
    }

    #[inline]
    fn is_none(value: &Self::Output) -> bool {
        *value == 0
    }

    #[inline]
    fn into_some(value: Self) -> Self::Output {
        value.get()
    }

    #[inline]
    fn from_some(value: Self::Output) -> Self {
        unsafe { Self::new_unchecked(value) }
    }
}

impl Niche for std::num::NonZeroIsize {
    type Output = isize;

    #[inline]
    fn none() -> Self::Output {
        0
    }

    #[inline]
    fn is_none(value: &Self::Output) -> bool {
        *value == 0
    }

    #[inline]
    fn into_some(value: Self) -> Self::Output {
        value.get()
    }

    #[inline]
    fn from_some(value: Self::Output) -> Self {
        unsafe { Self::new_unchecked(value) }
    }
}

impl Niche for std::num::NonZeroU8 {
    type Output = u8;

    #[inline]
    fn none() -> Self::Output {
        0
    }

    #[inline]
    fn is_none(value: &Self::Output) -> bool {
        *value == 0
    }

    #[inline]
    fn into_some(value: Self) -> Self::Output {
        value.get()
    }

    #[inline]
    fn from_some(value: Self::Output) -> Self {
        unsafe { Self::new_unchecked(value) }
    }
}

impl Niche for std::num::NonZeroU16 {
    type Output = u16;

    #[inline]
    fn none() -> Self::Output {
        0
    }

    #[inline]
    fn is_none(value: &Self::Output) -> bool {
        *value == 0
    }

    #[inline]
    fn into_some(value: Self) -> Self::Output {
        value.get()
    }

    #[inline]
    fn from_some(value: Self::Output) -> Self {
        unsafe { Self::new_unchecked(value) }
    }
}

impl Niche for std::num::NonZeroU32 {
    type Output = u32;

    #[inline]
    fn none() -> Self::Output {
        0
    }

    #[inline]
    fn is_none(value: &Self::Output) -> bool {
        *value == 0
    }

    #[inline]
    fn into_some(value: Self) -> Self::Output {
        value.get()
    }

    #[inline]
    fn from_some(value: Self::Output) -> Self {
        unsafe { Self::new_unchecked(value) }
    }
}

impl Niche for std::num::NonZeroU64 {
    type Output = u64;

    #[inline]
    fn none() -> Self::Output {
        0
    }

    #[inline]
    fn is_none(value: &Self::Output) -> bool {
        *value == 0
    }

    #[inline]
    fn into_some(value: Self) -> Self::Output {
        value.get()
    }

    #[inline]
    fn from_some(value: Self::Output) -> Self {
        unsafe { Self::new_unchecked(value) }
    }
}

impl Niche for std::num::NonZeroUsize {
    type Output = usize;

    #[inline]
    fn none() -> Self::Output {
        0
    }

    #[inline]
    fn is_none(value: &Self::Output) -> bool {
        *value == 0
    }

    #[inline]
    fn into_some(value: Self) -> Self::Output {
        value.get()
    }

    #[inline]
    fn from_some(value: Self::Output) -> Self {
        unsafe { Self::new_unchecked(value) }
    }
}
