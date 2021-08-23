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
