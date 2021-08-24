// -*- coding: utf-8 -*-
// ------------------------------------------------------------------------------------------------
// Copyright © 2021, Douglas Creager.
// Licensed under either of Apache License, Version 2.0, or MIT license, at your option.
// Please see the LICENSE-APACHE or LICENSE-MIT files in this distribution for license details.
// ------------------------------------------------------------------------------------------------

use std::num::NonZeroU32;

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
#[derive(Clone, Copy, Debug, Niche)]
struct TestStruct {
    a: NonZeroU32,
    #[niche]
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

// Same as above, but with a tuple struct.

#[repr(C)]
#[derive(Clone, Copy, Debug, Niche)]
struct TestTupleStruct(NonZeroU32, #[niche] NonZeroU32);

impl TestTupleStruct {
    fn new(a: u32, b: u32) -> TestTupleStruct {
        TestTupleStruct(NonZeroU32::new(a).unwrap(), NonZeroU32::new(b).unwrap())
    }
}

#[repr(C)]
#[derive(Debug)]
struct TestTupleStructRepr(u32, u32);

#[test]
fn can_option_tuple_structs() {
    let none = ControlledOption::<TestTupleStruct>::none();
    assert!(none.is_none());
    let none_repr: TestTupleStructRepr = unsafe { std::mem::transmute(none) };
    assert_eq!(none_repr.1, 0);

    let value = TestTupleStruct::new(75, 125);
    let some = ControlledOption::some(value);
    assert!(some.is_some());
    let some_repr: TestTupleStructRepr = unsafe { std::mem::transmute(some) };
    assert_eq!(some_repr.0, 75);
    assert_eq!(some_repr.1, 125);
}
