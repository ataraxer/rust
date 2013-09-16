// Copyright 2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// aux-build:issue_9188.rs
// xfail-fast windows doesn't like aux-build

extern mod issue_9188;

fn main() {
    let a = issue_9188::bar();
    let b = issue_9188::foo::<int>();
    assert_eq!(*a, *b);
}
