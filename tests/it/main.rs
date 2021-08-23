// -*- coding: utf-8 -*-
// ------------------------------------------------------------------------------------------------
// Copyright © 2021, Douglas Creager.
// Licensed under either of Apache License, Version 2.0, or MIT license, at your option.
// Please see the LICENSE-APACHE or LICENSE-MIT files in this distribution for license details.
// ------------------------------------------------------------------------------------------------

use std::mem::MaybeUninit;
use std::num::NonZeroU32;
use std::ptr::addr_of;
use std::ptr::addr_of_mut;

use controlled_option::ControlledOption;
use controlled_option::Niche;

#[test]
fn can_option_references() {
    let none = ControlledOption::<&u32>::none();
    assert!(none.is_none());
    // `None` references should be represented by the null pointer.
    assert_eq!(<&u32>::none(), std::ptr::null());

    let value = 75;
    let some = ControlledOption::some(&value);
    assert!(some.is_some());
    // `Some` references should be represented by (the pointer equivalent of) themselves.
    assert_eq!(<&u32>::from_some(&value), &value);
    assert_eq!(<&u32>::into_some(&value), &value);
}

#[test]
fn can_option_nonzeros() {
    let none = ControlledOption::from(NonZeroU32::new(0));
    assert!(none.is_none());
    // `None` non-zero values should be represented by 0.
    assert_eq!(NonZeroU32::none(), 0);

    let some = ControlledOption::from(NonZeroU32::new(75));
    assert!(some.is_some());
    // `Some` non-zero values should be represented by themselves.
    assert_eq!(NonZeroU32::from_some(75), NonZeroU32::new(75).unwrap());
}

// This is a struct that has two fields that have niche values available.  We'll explicitly choose
// to use the one from the second field as the niche for the struct as a whole.

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct TestStruct {
    a: NonZeroU32,
    b: NonZeroU32,
}

impl TestStruct {
    fn new(a: u32, b: u32) -> TestStruct {
        TestStruct {
            a: NonZeroU32::new(a).unwrap(),
            b: NonZeroU32::new(b).unwrap(),
        }
    }
}

fn fill_field_with_none<T>(field: *mut T)
where
    T: Niche,
{
    debug_assert!(std::alloc::Layout::new::<T>() == std::alloc::Layout::new::<T::Output>());
    let repr = field as *mut T::Output;
    unsafe { repr.write(T::none()) };
}

fn field_is_none<T>(field: *const T) -> bool
where
    T: Niche,
{
    debug_assert!(std::alloc::Layout::new::<T>() == std::alloc::Layout::new::<T::Output>());
    let repr = field as *const T::Output;
    T::is_none(unsafe { &*repr })
}

impl Niche for TestStruct {
    type Output = MaybeUninit<Self>;

    #[inline]
    fn none() -> Self::Output {
        let mut value = Self::Output::uninit();
        let ptr = value.as_mut_ptr();
        fill_field_with_none(unsafe { addr_of_mut!((*ptr).b) });
        value
    }

    #[inline]
    fn is_none(value: &Self::Output) -> bool {
        let ptr = value.as_ptr();
        field_is_none(unsafe { addr_of!((*ptr).b) })
    }

    #[inline]
    fn into_some(value: Self) -> Self::Output {
        MaybeUninit::new(value)
    }

    #[inline]
    fn from_some(value: Self::Output) -> Self {
        unsafe { value.assume_init() }
    }
}

#[repr(C)]
#[derive(Debug)]
struct TestStructRepr {
    a: u32,
    b: u32,
}

#[test]
fn can_option_structs() {
    let none = ControlledOption::<TestStruct>::none();
    assert!(none.is_none());
    let none_repr: TestStructRepr = unsafe { std::mem::transmute(none) };
    assert_eq!(none_repr.b, 0);

    let value = TestStruct::new(75, 125);
    let some = ControlledOption::some(value);
    assert!(some.is_some());
    let some_repr: TestStructRepr = unsafe { std::mem::transmute(some) };
    assert_eq!(some_repr.a, 75);
    assert_eq!(some_repr.b, 125);
}
