// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// ignore-lexer-test FIXME #15883

use borrow::BorrowFrom;
use clone::Clone;
use cmp::{Eq, Equiv, PartialEq};
use core::kinds::Sized;
use default::Default;
use fmt::Show;
use fmt;
use hash::{Hash, Hasher, RandomSipHasher};
use iter::{Iterator, IteratorExt, IteratorCloneExt, FromIterator, Map, Chain, Extend};
use ops::{BitOr, BitAnd, BitXor, Sub};
use option::Option::{Some, None, mod};
use result::Result::{Ok, Err};

use super::map::{mod, HashMap, Keys, INITIAL_CAPACITY};

// Future Optimization (FIXME!)
// =============================
//
// Iteration over zero sized values is a noop. There is no need
// for `bucket.val` in the case of HashSet. I suppose we would need HKT
// to get rid of it properly.

/// An implementation of a hash set using the underlying representation of a
/// HashMap where the value is (). As with the `HashMap` type, a `HashSet`
/// requires that the elements implement the `Eq` and `Hash` traits.
///
/// # Example
///
/// ```
/// use std::collections::HashSet;
/// // Type inference lets us omit an explicit type signature (which
/// // would be `HashSet<&str>` in this example).
/// let mut books = HashSet::new();
///
/// // Add some books.
/// books.insert("A Dance With Dragons");
/// books.insert("To Kill a Mockingbird");
/// books.insert("The Odyssey");
/// books.insert("The Great Gatsby");
///
/// // Check for a specific one.
/// if !books.contains(&("The Winds of Winter")) {
///     println!("We have {} books, but The Winds of Winter ain't one.",
///              books.len());
/// }
///
/// // Remove a book.
/// books.remove(&"The Odyssey");
///
/// // Iterate over everything.
/// for book in books.iter() {
///     println!("{}", *book);
/// }
/// ```
///
/// The easiest way to use `HashSet` with a custom type is to derive
/// `Eq` and `Hash`. We must also derive `PartialEq`, this will in the
/// future be implied by `Eq`.
///
/// ```
/// use std::collections::HashSet;
/// #[deriving(Hash, Eq, PartialEq, Show)]
/// struct Viking<'a> {
///     name: &'a str,
///     power: uint,
/// }
///
/// let mut vikings = HashSet::new();
///
/// vikings.insert(Viking { name: "Einar", power: 9u });
/// vikings.insert(Viking { name: "Einar", power: 9u });
/// vikings.insert(Viking { name: "Olaf", power: 4u });
/// vikings.insert(Viking { name: "Harald", power: 8u });
///
/// // Use derived implementation to print the vikings.
/// for x in vikings.iter() {
///     println!("{}", x);
/// }
/// ```
#[deriving(Clone)]
#[stable]
pub struct HashSet<T, H = RandomSipHasher> {
    map: HashMap<T, (), H>
}

impl<T: Hash + Eq> HashSet<T, RandomSipHasher> {
    /// Create an empty HashSet.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    /// let mut set: HashSet<int> = HashSet::new();
    /// ```
    #[inline]
    #[stable]
    pub fn new() -> HashSet<T, RandomSipHasher> {
        HashSet::with_capacity(INITIAL_CAPACITY)
    }

    /// Create an empty HashSet with space for at least `n` elements in
    /// the hash table.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    /// let mut set: HashSet<int> = HashSet::with_capacity(10);
    /// ```
    #[inline]
    #[stable]
    pub fn with_capacity(capacity: uint) -> HashSet<T, RandomSipHasher> {
        HashSet { map: HashMap::with_capacity(capacity) }
    }
}

impl<T: Eq + Hash<S>, S, H: Hasher<S>> HashSet<T, H> {
    /// Creates a new empty hash set which will use the given hasher to hash
    /// keys.
    ///
    /// The hash set is also created with the default initial capacity.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use std::hash::sip::SipHasher;
    ///
    /// let h = SipHasher::new();
    /// let mut set = HashSet::with_hasher(h);
    /// set.insert(2u);
    /// ```
    #[inline]
    #[unstable = "hasher stuff is unclear"]
    pub fn with_hasher(hasher: H) -> HashSet<T, H> {
        HashSet::with_capacity_and_hasher(INITIAL_CAPACITY, hasher)
    }

    /// Create an empty HashSet with space for at least `capacity`
    /// elements in the hash table, using `hasher` to hash the keys.
    ///
    /// Warning: `hasher` is normally randomly generated, and
    /// is designed to allow `HashSet`s to be resistant to attacks that
    /// cause many collisions and very poor performance. Setting it
    /// manually using this function can expose a DoS attack vector.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use std::hash::sip::SipHasher;
    ///
    /// let h = SipHasher::new();
    /// let mut set = HashSet::with_capacity_and_hasher(10u, h);
    /// set.insert(1i);
    /// ```
    #[inline]
    #[unstable = "hasher stuff is unclear"]
    pub fn with_capacity_and_hasher(capacity: uint, hasher: H) -> HashSet<T, H> {
        HashSet { map: HashMap::with_capacity_and_hasher(capacity, hasher) }
    }

    /// Returns the number of elements the set can hold without reallocating.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    /// let set: HashSet<int> = HashSet::with_capacity(100);
    /// assert!(set.capacity() >= 100);
    /// ```
    #[inline]
    #[stable]
    pub fn capacity(&self) -> uint {
        self.map.capacity()
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the `HashSet`. The collection may reserve more space to avoid
    /// frequent reallocations.
    ///
    /// # Panics
    ///
    /// Panics if the new allocation size overflows `uint`.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    /// let mut set: HashSet<int> = HashSet::new();
    /// set.reserve(10);
    /// ```
    #[stable]
    pub fn reserve(&mut self, additional: uint) {
        self.map.reserve(additional)
    }

    /// Shrinks the capacity of the set as much as possible. It will drop
    /// down as much as possible while maintaining the internal rules
    /// and possibly leaving some space in accordance with the resize policy.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// let mut set: HashSet<int> = HashSet::with_capacity(100);
    /// set.insert(1);
    /// set.insert(2);
    /// assert!(set.capacity() >= 100);
    /// set.shrink_to_fit();
    /// assert!(set.capacity() >= 2);
    /// ```
    #[stable]
    pub fn shrink_to_fit(&mut self) {
        self.map.shrink_to_fit()
    }

    /// Deprecated: use `contains` and `BorrowFrom`.
    #[deprecated = "use contains and BorrowFrom"]
    #[allow(deprecated)]
    pub fn contains_equiv<Sized? Q: Hash<S> + Equiv<T>>(&self, value: &Q) -> bool {
      self.map.contains_key_equiv(value)
    }

    /// An iterator visiting all elements in arbitrary order.
    /// Iterator element type is &'a T.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    /// let mut set = HashSet::new();
    /// set.insert("a");
    /// set.insert("b");
    ///
    /// // Will print in an arbitrary order.
    /// for x in set.iter() {
    ///     println!("{}", x);
    /// }
    /// ```
    #[stable]
    pub fn iter(&self) -> Iter<T> {
        Iter { iter: self.map.keys() }
    }

    /// Creates a consuming iterator, that is, one that moves each value out
    /// of the set in arbitrary order. The set cannot be used after calling
    /// this.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    /// let mut set = HashSet::new();
    /// set.insert("a".to_string());
    /// set.insert("b".to_string());
    ///
    /// // Not possible to collect to a Vec<String> with a regular `.iter()`.
    /// let v: Vec<String> = set.into_iter().collect();
    ///
    /// // Will print in an arbitrary order.
    /// for x in v.iter() {
    ///     println!("{}", x);
    /// }
    /// ```
    #[stable]
    pub fn into_iter(self) -> IntoIter<T> {
        fn first<A, B>((a, _): (A, B)) -> A { a }
        let first: fn((T, ())) -> T = first;

        IntoIter { iter: self.map.into_iter().map(first) }
    }

    /// Visit the values representing the difference.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    /// let a: HashSet<int> = [1i, 2, 3].iter().map(|&x| x).collect();
    /// let b: HashSet<int> = [4i, 2, 3, 4].iter().map(|&x| x).collect();
    ///
    /// // Can be seen as `a - b`.
    /// for x in a.difference(&b) {
    ///     println!("{}", x); // Print 1
    /// }
    ///
    /// let diff: HashSet<int> = a.difference(&b).map(|&x| x).collect();
    /// assert_eq!(diff, [1i].iter().map(|&x| x).collect());
    ///
    /// // Note that difference is not symmetric,
    /// // and `b - a` means something else:
    /// let diff: HashSet<int> = b.difference(&a).map(|&x| x).collect();
    /// assert_eq!(diff, [4i].iter().map(|&x| x).collect());
    /// ```
    #[stable]
    pub fn difference<'a>(&'a self, other: &'a HashSet<T, H>) -> Difference<'a, T, H> {
        Difference {
            iter: self.iter(),
            other: other,
        }
    }

    /// Visit the values representing the symmetric difference.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    /// let a: HashSet<int> = [1i, 2, 3].iter().map(|&x| x).collect();
    /// let b: HashSet<int> = [4i, 2, 3, 4].iter().map(|&x| x).collect();
    ///
    /// // Print 1, 4 in arbitrary order.
    /// for x in a.symmetric_difference(&b) {
    ///     println!("{}", x);
    /// }
    ///
    /// let diff1: HashSet<int> = a.symmetric_difference(&b).map(|&x| x).collect();
    /// let diff2: HashSet<int> = b.symmetric_difference(&a).map(|&x| x).collect();
    ///
    /// assert_eq!(diff1, diff2);
    /// assert_eq!(diff1, [1i, 4].iter().map(|&x| x).collect());
    /// ```
    #[stable]
    pub fn symmetric_difference<'a>(&'a self, other: &'a HashSet<T, H>)
        -> SymmetricDifference<'a, T, H> {
        SymmetricDifference { iter: self.difference(other).chain(other.difference(self)) }
    }

    /// Visit the values representing the intersection.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    /// let a: HashSet<int> = [1i, 2, 3].iter().map(|&x| x).collect();
    /// let b: HashSet<int> = [4i, 2, 3, 4].iter().map(|&x| x).collect();
    ///
    /// // Print 2, 3 in arbitrary order.
    /// for x in a.intersection(&b) {
    ///     println!("{}", x);
    /// }
    ///
    /// let diff: HashSet<int> = a.intersection(&b).map(|&x| x).collect();
    /// assert_eq!(diff, [2i, 3].iter().map(|&x| x).collect());
    /// ```
    #[stable]
    pub fn intersection<'a>(&'a self, other: &'a HashSet<T, H>) -> Intersection<'a, T, H> {
        Intersection {
            iter: self.iter(),
            other: other,
        }
    }

    /// Visit the values representing the union.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    /// let a: HashSet<int> = [1i, 2, 3].iter().map(|&x| x).collect();
    /// let b: HashSet<int> = [4i, 2, 3, 4].iter().map(|&x| x).collect();
    ///
    /// // Print 1, 2, 3, 4 in arbitrary order.
    /// for x in a.union(&b) {
    ///     println!("{}", x);
    /// }
    ///
    /// let diff: HashSet<int> = a.union(&b).map(|&x| x).collect();
    /// assert_eq!(diff, [1i, 2, 3, 4].iter().map(|&x| x).collect());
    /// ```
    #[stable]
    pub fn union<'a>(&'a self, other: &'a HashSet<T, H>) -> Union<'a, T, H> {
        Union { iter: self.iter().chain(other.difference(self)) }
    }

    /// Return the number of elements in the set
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// let mut v = HashSet::new();
    /// assert_eq!(v.len(), 0);
    /// v.insert(1u);
    /// assert_eq!(v.len(), 1);
    /// ```
    #[stable]
    pub fn len(&self) -> uint { self.map.len() }

    /// Returns true if the set contains no elements
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// let mut v = HashSet::new();
    /// assert!(v.is_empty());
    /// v.insert(1u);
    /// assert!(!v.is_empty());
    /// ```
    #[stable]
    pub fn is_empty(&self) -> bool { self.map.len() == 0 }

    /// Clears the set, returning all elements in an iterator.
    #[inline]
    #[unstable = "matches collection reform specification, waiting for dust to settle"]
    pub fn drain(&mut self) -> Drain<T> {
        fn first<A, B>((a, _): (A, B)) -> A { a }
        let first: fn((T, ())) -> T = first; // coerce to fn pointer

        Drain { iter: self.map.drain().map(first) }
    }

    /// Clears the set, removing all values.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// let mut v = HashSet::new();
    /// v.insert(1u);
    /// v.clear();
    /// assert!(v.is_empty());
    /// ```
    #[stable]
    pub fn clear(&mut self) { self.map.clear() }

    /// Returns `true` if the set contains a value.
    ///
    /// The value may be any borrowed form of the set's value type, but
    /// `Hash` and `Eq` on the borrowed form *must* match those for
    /// the value type.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// let set: HashSet<uint> = [1, 2, 3].iter().map(|&x| x).collect();
    /// assert_eq!(set.contains(&1), true);
    /// assert_eq!(set.contains(&4), false);
    /// ```
    #[stable]
    pub fn contains<Sized? Q>(&self, value: &Q) -> bool
        where Q: BorrowFrom<T> + Hash<S> + Eq
    {
        self.map.contains_key(value)
    }

    /// Returns `true` if the set has no elements in common with `other`.
    /// This is equivalent to checking for an empty intersection.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// let a: HashSet<uint> = [1, 2, 3].iter().map(|&x| x).collect();
    /// let mut b: HashSet<uint> = HashSet::new();
    ///
    /// assert_eq!(a.is_disjoint(&b), true);
    /// b.insert(4);
    /// assert_eq!(a.is_disjoint(&b), true);
    /// b.insert(1);
    /// assert_eq!(a.is_disjoint(&b), false);
    /// ```
    #[stable]
    pub fn is_disjoint(&self, other: &HashSet<T, H>) -> bool {
        self.iter().all(|v| !other.contains(v))
    }

    /// Returns `true` if the set is a subset of another.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// let sup: HashSet<uint> = [1, 2, 3].iter().map(|&x| x).collect();
    /// let mut set: HashSet<uint> = HashSet::new();
    ///
    /// assert_eq!(set.is_subset(&sup), true);
    /// set.insert(2);
    /// assert_eq!(set.is_subset(&sup), true);
    /// set.insert(4);
    /// assert_eq!(set.is_subset(&sup), false);
    /// ```
    #[stable]
    pub fn is_subset(&self, other: &HashSet<T, H>) -> bool {
        self.iter().all(|v| other.contains(v))
    }

    /// Returns `true` if the set is a superset of another.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// let sub: HashSet<uint> = [1, 2].iter().map(|&x| x).collect();
    /// let mut set: HashSet<uint> = HashSet::new();
    ///
    /// assert_eq!(set.is_superset(&sub), false);
    ///
    /// set.insert(0);
    /// set.insert(1);
    /// assert_eq!(set.is_superset(&sub), false);
    ///
    /// set.insert(2);
    /// assert_eq!(set.is_superset(&sub), true);
    /// ```
    #[inline]
    #[stable]
    pub fn is_superset(&self, other: &HashSet<T, H>) -> bool {
        other.is_subset(self)
    }

    /// Adds a value to the set. Returns `true` if the value was not already
    /// present in the set.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// let mut set = HashSet::new();
    ///
    /// assert_eq!(set.insert(2u), true);
    /// assert_eq!(set.insert(2), false);
    /// assert_eq!(set.len(), 1);
    /// ```
    #[stable]
    pub fn insert(&mut self, value: T) -> bool { self.map.insert(value, ()).is_none() }

    /// Removes a value from the set. Returns `true` if the value was
    /// present in the set.
    ///
    /// The value may be any borrowed form of the set's value type, but
    /// `Hash` and `Eq` on the borrowed form *must* match those for
    /// the value type.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// let mut set = HashSet::new();
    ///
    /// set.insert(2u);
    /// assert_eq!(set.remove(&2), true);
    /// assert_eq!(set.remove(&2), false);
    /// ```
    #[stable]
    pub fn remove<Sized? Q>(&mut self, value: &Q) -> bool
        where Q: BorrowFrom<T> + Hash<S> + Eq
    {
        self.map.remove(value).is_some()
    }
}

#[stable]
impl<T: Eq + Hash<S>, S, H: Hasher<S>> PartialEq for HashSet<T, H> {
    fn eq(&self, other: &HashSet<T, H>) -> bool {
        if self.len() != other.len() { return false; }

        self.iter().all(|key| other.contains(key))
    }
}

#[stable]
impl<T: Eq + Hash<S>, S, H: Hasher<S>> Eq for HashSet<T, H> {}

#[stable]
impl<T: Eq + Hash<S> + fmt::Show, S, H: Hasher<S>> fmt::Show for HashSet<T, H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{{"));

        for (i, x) in self.iter().enumerate() {
            if i != 0 { try!(write!(f, ", ")); }
            try!(write!(f, "{}", *x));
        }

        write!(f, "}}")
    }
}

#[stable]
impl<T: Eq + Hash<S>, S, H: Hasher<S> + Default> FromIterator<T> for HashSet<T, H> {
    fn from_iter<I: Iterator<T>>(iter: I) -> HashSet<T, H> {
        let lower = iter.size_hint().0;
        let mut set = HashSet::with_capacity_and_hasher(lower, Default::default());
        set.extend(iter);
        set
    }
}

#[stable]
impl<T: Eq + Hash<S>, S, H: Hasher<S> + Default> Extend<T> for HashSet<T, H> {
    fn extend<I: Iterator<T>>(&mut self, mut iter: I) {
        for k in iter {
            self.insert(k);
        }
    }
}

#[stable]
impl<T: Eq + Hash<S>, S, H: Hasher<S> + Default> Default for HashSet<T, H> {
    #[stable]
    fn default() -> HashSet<T, H> {
        HashSet::with_hasher(Default::default())
    }
}

#[stable]
impl<'a, 'b, T: Eq + Hash<S> + Clone, S, H: Hasher<S> + Default>
BitOr<&'b HashSet<T, H>, HashSet<T, H>> for &'a HashSet<T, H> {
    /// Returns the union of `self` and `rhs` as a new `HashSet<T, H>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// let a: HashSet<int> = vec![1, 2, 3].into_iter().collect();
    /// let b: HashSet<int> = vec![3, 4, 5].into_iter().collect();
    ///
    /// let set: HashSet<int> = &a | &b;
    ///
    /// let mut i = 0;
    /// let expected = [1, 2, 3, 4, 5];
    /// for x in set.iter() {
    ///     assert!(expected.contains(x));
    ///     i += 1;
    /// }
    /// assert_eq!(i, expected.len());
    /// ```
    fn bitor(self, rhs: &HashSet<T, H>) -> HashSet<T, H> {
        self.union(rhs).cloned().collect()
    }
}

#[stable]
impl<'a, 'b, T: Eq + Hash<S> + Clone, S, H: Hasher<S> + Default>
BitAnd<&'b HashSet<T, H>, HashSet<T, H>> for &'a HashSet<T, H> {
    /// Returns the intersection of `self` and `rhs` as a new `HashSet<T, H>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// let a: HashSet<int> = vec![1, 2, 3].into_iter().collect();
    /// let b: HashSet<int> = vec![2, 3, 4].into_iter().collect();
    ///
    /// let set: HashSet<int> = &a & &b;
    ///
    /// let mut i = 0;
    /// let expected = [2, 3];
    /// for x in set.iter() {
    ///     assert!(expected.contains(x));
    ///     i += 1;
    /// }
    /// assert_eq!(i, expected.len());
    /// ```
    fn bitand(self, rhs: &HashSet<T, H>) -> HashSet<T, H> {
        self.intersection(rhs).cloned().collect()
    }
}

#[stable]
impl<'a, 'b, T: Eq + Hash<S> + Clone, S, H: Hasher<S> + Default>
BitXor<&'b HashSet<T, H>, HashSet<T, H>> for &'a HashSet<T, H> {
    /// Returns the symmetric difference of `self` and `rhs` as a new `HashSet<T, H>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// let a: HashSet<int> = vec![1, 2, 3].into_iter().collect();
    /// let b: HashSet<int> = vec![3, 4, 5].into_iter().collect();
    ///
    /// let set: HashSet<int> = &a ^ &b;
    ///
    /// let mut i = 0;
    /// let expected = [1, 2, 4, 5];
    /// for x in set.iter() {
    ///     assert!(expected.contains(x));
    ///     i += 1;
    /// }
    /// assert_eq!(i, expected.len());
    /// ```
    fn bitxor(self, rhs: &HashSet<T, H>) -> HashSet<T, H> {
        self.symmetric_difference(rhs).cloned().collect()
    }
}

#[stable]
impl<'a, 'b, T: Eq + Hash<S> + Clone, S, H: Hasher<S> + Default>
Sub<&'b HashSet<T, H>, HashSet<T, H>> for &'a HashSet<T, H> {
    /// Returns the difference of `self` and `rhs` as a new `HashSet<T, H>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// let a: HashSet<int> = vec![1, 2, 3].into_iter().collect();
    /// let b: HashSet<int> = vec![3, 4, 5].into_iter().collect();
    ///
    /// let set: HashSet<int> = &a - &b;
    ///
    /// let mut i = 0;
    /// let expected = [1, 2];
    /// for x in set.iter() {
    ///     assert!(expected.contains(x));
    ///     i += 1;
    /// }
    /// assert_eq!(i, expected.len());
    /// ```
    fn sub(self, rhs: &HashSet<T, H>) -> HashSet<T, H> {
        self.difference(rhs).cloned().collect()
    }
}

/// HashSet iterator
#[stable]
pub struct Iter<'a, K: 'a> {
    iter: Keys<'a, K, ()>
}

/// HashSet move iterator
#[stable]
pub struct IntoIter<K> {
    iter: Map<(K, ()), K, map::IntoIter<K, ()>, fn((K, ())) -> K>
}

/// HashSet drain iterator
#[stable]
pub struct Drain<'a, K: 'a> {
    iter: Map<(K, ()), K, map::Drain<'a, K, ()>, fn((K, ())) -> K>,
}

/// Intersection iterator
#[stable]
pub struct Intersection<'a, T: 'a, H: 'a> {
    // iterator of the first set
    iter: Iter<'a, T>,
    // the second set
    other: &'a HashSet<T, H>,
}

/// Difference iterator
#[stable]
pub struct Difference<'a, T: 'a, H: 'a> {
    // iterator of the first set
    iter: Iter<'a, T>,
    // the second set
    other: &'a HashSet<T, H>,
}

/// Symmetric difference iterator.
#[stable]
pub struct SymmetricDifference<'a, T: 'a, H: 'a> {
    iter: Chain<Difference<'a, T, H>, Difference<'a, T, H>>
}

/// Set union iterator.
#[stable]
pub struct Union<'a, T: 'a, H: 'a> {
    iter: Chain<Iter<'a, T>, Difference<'a, T, H>>
}

#[stable]
impl<'a, K> Iterator<&'a K> for Iter<'a, K> {
    fn next(&mut self) -> Option<&'a K> { self.iter.next() }
    fn size_hint(&self) -> (uint, Option<uint>) { self.iter.size_hint() }
}

#[stable]
impl<K> Iterator<K> for IntoIter<K> {
    fn next(&mut self) -> Option<K> { self.iter.next() }
    fn size_hint(&self) -> (uint, Option<uint>) { self.iter.size_hint() }
}

#[stable]
impl<'a, K: 'a> Iterator<K> for Drain<'a, K> {
    fn next(&mut self) -> Option<K> { self.iter.next() }
    fn size_hint(&self) -> (uint, Option<uint>) { self.iter.size_hint() }
}

#[stable]
impl<'a, T, S, H> Iterator<&'a T> for Intersection<'a, T, H>
    where T: Eq + Hash<S>, H: Hasher<S>
{
    fn next(&mut self) -> Option<&'a T> {
        loop {
            match self.iter.next() {
                None => return None,
                Some(elt) => if self.other.contains(elt) {
                    return Some(elt)
                },
            }
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper)
    }
}

#[stable]
impl<'a, T, S, H> Iterator<&'a T> for Difference<'a, T, H>
    where T: Eq + Hash<S>, H: Hasher<S>
{
    fn next(&mut self) -> Option<&'a T> {
        loop {
            match self.iter.next() {
                None => return None,
                Some(elt) => if !self.other.contains(elt) {
                    return Some(elt)
                },
            }
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper)
    }
}

#[stable]
impl<'a, T, S, H> Iterator<&'a T> for SymmetricDifference<'a, T, H>
    where T: Eq + Hash<S>, H: Hasher<S>
{
    fn next(&mut self) -> Option<&'a T> { self.iter.next() }
    fn size_hint(&self) -> (uint, Option<uint>) { self.iter.size_hint() }
}

#[stable]
impl<'a, T, S, H> Iterator<&'a T> for Union<'a, T, H>
    where T: Eq + Hash<S>, H: Hasher<S>
{
    fn next(&mut self) -> Option<&'a T> { self.iter.next() }
    fn size_hint(&self) -> (uint, Option<uint>) { self.iter.size_hint() }
}

#[cfg(test)]
mod test_set {
    use prelude::v1::*;

    use super::HashSet;

    #[test]
    fn test_disjoint() {
        let mut xs = HashSet::new();
        let mut ys = HashSet::new();
        assert!(xs.is_disjoint(&ys));
        assert!(ys.is_disjoint(&xs));
        assert!(xs.insert(5i));
        assert!(ys.insert(11i));
        assert!(xs.is_disjoint(&ys));
        assert!(ys.is_disjoint(&xs));
        assert!(xs.insert(7));
        assert!(xs.insert(19));
        assert!(xs.insert(4));
        assert!(ys.insert(2));
        assert!(ys.insert(-11));
        assert!(xs.is_disjoint(&ys));
        assert!(ys.is_disjoint(&xs));
        assert!(ys.insert(7));
        assert!(!xs.is_disjoint(&ys));
        assert!(!ys.is_disjoint(&xs));
    }

    #[test]
    fn test_subset_and_superset() {
        let mut a = HashSet::new();
        assert!(a.insert(0i));
        assert!(a.insert(5));
        assert!(a.insert(11));
        assert!(a.insert(7));

        let mut b = HashSet::new();
        assert!(b.insert(0i));
        assert!(b.insert(7));
        assert!(b.insert(19));
        assert!(b.insert(250));
        assert!(b.insert(11));
        assert!(b.insert(200));

        assert!(!a.is_subset(&b));
        assert!(!a.is_superset(&b));
        assert!(!b.is_subset(&a));
        assert!(!b.is_superset(&a));

        assert!(b.insert(5));

        assert!(a.is_subset(&b));
        assert!(!a.is_superset(&b));
        assert!(!b.is_subset(&a));
        assert!(b.is_superset(&a));
    }

    #[test]
    fn test_iterate() {
        let mut a = HashSet::new();
        for i in range(0u, 32) {
            assert!(a.insert(i));
        }
        let mut observed: u32 = 0;
        for k in a.iter() {
            observed |= 1 << *k;
        }
        assert_eq!(observed, 0xFFFF_FFFF);
    }

    #[test]
    fn test_intersection() {
        let mut a = HashSet::new();
        let mut b = HashSet::new();

        assert!(a.insert(11i));
        assert!(a.insert(1));
        assert!(a.insert(3));
        assert!(a.insert(77));
        assert!(a.insert(103));
        assert!(a.insert(5));
        assert!(a.insert(-5));

        assert!(b.insert(2i));
        assert!(b.insert(11));
        assert!(b.insert(77));
        assert!(b.insert(-9));
        assert!(b.insert(-42));
        assert!(b.insert(5));
        assert!(b.insert(3));

        let mut i = 0;
        let expected = [3, 5, 11, 77];
        for x in a.intersection(&b) {
            assert!(expected.contains(x));
            i += 1
        }
        assert_eq!(i, expected.len());
    }

    #[test]
    fn test_difference() {
        let mut a = HashSet::new();
        let mut b = HashSet::new();

        assert!(a.insert(1i));
        assert!(a.insert(3));
        assert!(a.insert(5));
        assert!(a.insert(9));
        assert!(a.insert(11));

        assert!(b.insert(3i));
        assert!(b.insert(9));

        let mut i = 0;
        let expected = [1, 5, 11];
        for x in a.difference(&b) {
            assert!(expected.contains(x));
            i += 1
        }
        assert_eq!(i, expected.len());
    }

    #[test]
    fn test_symmetric_difference() {
        let mut a = HashSet::new();
        let mut b = HashSet::new();

        assert!(a.insert(1i));
        assert!(a.insert(3));
        assert!(a.insert(5));
        assert!(a.insert(9));
        assert!(a.insert(11));

        assert!(b.insert(-2i));
        assert!(b.insert(3));
        assert!(b.insert(9));
        assert!(b.insert(14));
        assert!(b.insert(22));

        let mut i = 0;
        let expected = [-2, 1, 5, 11, 14, 22];
        for x in a.symmetric_difference(&b) {
            assert!(expected.contains(x));
            i += 1
        }
        assert_eq!(i, expected.len());
    }

    #[test]
    fn test_union() {
        let mut a = HashSet::new();
        let mut b = HashSet::new();

        assert!(a.insert(1i));
        assert!(a.insert(3));
        assert!(a.insert(5));
        assert!(a.insert(9));
        assert!(a.insert(11));
        assert!(a.insert(16));
        assert!(a.insert(19));
        assert!(a.insert(24));

        assert!(b.insert(-2i));
        assert!(b.insert(1));
        assert!(b.insert(5));
        assert!(b.insert(9));
        assert!(b.insert(13));
        assert!(b.insert(19));

        let mut i = 0;
        let expected = [-2, 1, 3, 5, 9, 11, 13, 16, 19, 24];
        for x in a.union(&b) {
            assert!(expected.contains(x));
            i += 1
        }
        assert_eq!(i, expected.len());
    }

    #[test]
    fn test_from_iter() {
        let xs = [1i, 2, 3, 4, 5, 6, 7, 8, 9];

        let set: HashSet<int> = xs.iter().map(|&x| x).collect();

        for x in xs.iter() {
            assert!(set.contains(x));
        }
    }

    #[test]
    fn test_move_iter() {
        let hs = {
            let mut hs = HashSet::new();

            hs.insert('a');
            hs.insert('b');

            hs
        };

        let v = hs.into_iter().collect::<Vec<char>>();
        assert!(['a', 'b'] == v || ['b', 'a'] == v);
    }

    #[test]
    fn test_eq() {
        // These constants once happened to expose a bug in insert().
        // I'm keeping them around to prevent a regression.
        let mut s1 = HashSet::new();

        s1.insert(1i);
        s1.insert(2);
        s1.insert(3);

        let mut s2 = HashSet::new();

        s2.insert(1i);
        s2.insert(2);

        assert!(s1 != s2);

        s2.insert(3);

        assert_eq!(s1, s2);
    }

    #[test]
    fn test_show() {
        let mut set: HashSet<int> = HashSet::new();
        let empty: HashSet<int> = HashSet::new();

        set.insert(1i);
        set.insert(2);

        let set_str = format!("{}", set);

        assert!(set_str == "{1, 2}" || set_str == "{2, 1}");
        assert_eq!(format!("{}", empty), "{}");
    }

    #[test]
    fn test_trivial_drain() {
        let mut s = HashSet::<int>::new();
        for _ in s.drain() {}
        assert!(s.is_empty());
        drop(s);

        let mut s = HashSet::<int>::new();
        drop(s.drain());
        assert!(s.is_empty());
    }

    #[test]
    fn test_drain() {
        let mut s: HashSet<int> = range(1, 100).collect();

        // try this a bunch of times to make sure we don't screw up internal state.
        for _ in range(0i, 20) {
            assert_eq!(s.len(), 99);

            {
                let mut last_i = 0;
                let mut d = s.drain();
                for (i, x) in d.by_ref().take(50).enumerate() {
                    last_i = i;
                    assert!(x != 0);
                }
                assert_eq!(last_i, 49);
            }

            for _ in s.iter() { panic!("s should be empty!"); }

            // reset to try again.
            s.extend(range(1, 100));
        }
    }
}
