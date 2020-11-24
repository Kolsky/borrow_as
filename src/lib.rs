//! # Borrow As
//! Partial struct borrowing made easy, including splitting borrows.
//! # Example
//! ```
//! use std::borrow::Borrow;
//! use borrow_as::*;
//!
//! struct X {
//!     s: String,
//!     v: Vec<u128>,
//!     i: i8,
//!     x: u32,
//!     f: Box<dyn Fn() -> i32>,
//! }
//!
//! impl Default for X {
//!     fn default() -> Self {
//!         Self {
//!             s: String::from("No string for you"),
//!             v: vec![1, 2, 3],
//!             i: 0,
//!             x: 9,
//!             f: Box::new(|| 0),
//!         }
//!     }
//! }
//!
//! impl X {
//!     fn construct_a<'a>(s: &'a String,
//!                        v: &'a Vec<u128>)
//!                        -> LifeRef<'a, A> {
//!         LifeRef::
//!             wrap_ref(s.as_str())
//!             .add_ref(v.as_slice())
//!             .map_life(|(s, v)| A { s, v })
//!     }
//!
//!     fn construct_b<'a>(i: &'a mut i8,
//!                        x: u32,
//!                        f: &'a (dyn Fn() -> i32 + 'static))
//!                        -> LifeRef<'a, B> {
//!         LifeRef::
//!             wrap_mut(i)
//!             .add_ref(f)
//!             .map_life(|(i, f)| B { i, x, f })
//!     }
//!
//!     pub fn get_a(&self) -> LifeRef<'_, A> {
//!         let Self { s, v, .. } = self;
//!         Self::construct_a(s, v)
//!     }
//!
//!     pub fn get_b(&mut self) -> LifeRef<'_, B> {
//!         let Self { i, x, .. } = self;
//!         Self::construct_b(i, *x, self.f.as_ref())
//!     }
//!
//!     pub fn get_ab(&mut self) -> LifeRef<'_, (A, B)> {
//!         let Self { s, v, i, x, .. } = self;
//!         let a = Self::construct_a(s, v);
//!         let b = Self::construct_b(i, *x, self.f.as_ref());
//!         a.wrap_life().add_life(b)
//!     }
//!
//!     pub fn get_c(&mut self) -> LifeRef<'_, C> {
//!         let Self { f, v, i, .. } = self;
//!         LifeRef::
//!             wrap_mut(f)
//!             .add_mut(v.as_mut_slice())
//!             .add_ref(i)
//!             .map_life(|(f, v, i)| C { f, v, i })
//!     }
//! }
//!
//! struct A {
//!     pub s: Ref<str>,
//!     pub v: Ref<[u128]>,
//! }
//!
//! pub struct B {
//!     pub i: Mut<i8>,
//!     pub x: u32,
//!     pub f: Ref<dyn Fn() -> i32>,
//! }
//!
//! pub struct C {
//!     pub v: Mut<[u128]>,
//!     pub i: Ref<i8>,
//!     pub f: Mut<Box<dyn Fn() -> i32>>,
//! }
//!
//! let mut x = X::default();
//!
//! let a = x.get_a();
//! assert_eq!(a.s, "No string for you");
//! assert_eq!(a.v, [1, 2, 3]);
//!
//! let b = x.get_b();
//! let b: &B = b.borrow();
//! assert_eq!(b.i.get(), 0);
//! assert_eq!(b.x, 9);
//! assert_eq!((b.f)(), 0);
//!
//! b.i.set(1);
//! let c = x.get_c();
//! let c: &C = c.borrow();
//! assert_eq!(c.i, &1);
//!
//! c.f.set(Box::new(|| { 8 }));
//! let v = c.v.as_slice_of_cells();
//! v[2].set(4);
//! let ab = x.get_ab();
//! assert_eq!(ab.0.v, [1, 2, 4]);
//! assert_eq!((ab.1.f)(), 8);
//!
//! assert_eq!(x.s, "No string for you");
//! assert_eq!(x.v, [1, 2, 4]);
//! assert_eq!(x.i, 1);
//! assert_eq!(x.x, 9);
//! assert_eq!((x.f)(), 8);
#![cfg_attr(not(test), no_std)]
use core::fmt;
use core::ops::Deref;
use core::borrow::Borrow;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::cell::Cell;
use tuple_utils::Append;

/// Container for value which remains valid over specified lifetime.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)]
#[repr(transparent)]
pub struct LifeRef<'a, T>{
    inner: T,
    phantom: PhantomData<&'a T>,
}

impl<'a, T: ?Sized> LifeRef<'a, (Ref<T>,)> {
    /// Wraps immutable reference with inner value represented as 1-tuple for chaining with other methods.
    /// # Example
    /// ```
    /// let s = String::from("Referenced");
    /// let r = borrow_as::LifeRef::wrap_ref(&s[..3]);
    /// assert_eq!(r.0, "Ref");
    pub fn wrap_ref(r: &'a T) -> Self {
        Self {
            inner: (Ref(r),),
            phantom: PhantomData,
        }
    }
}

impl<'a, T: ?Sized> LifeRef<'a, (Mut<T>,)> {
    /// Wraps mutable reference with inner value represented as 1-tuple for chaining with other methods.
    /// # Example
    /// ```
    /// let mut v = vec![1, 2, 3];
    /// let r_mut = borrow_as::LifeRef::wrap_mut(v.as_mut_slice());
    /// let v_mut = r_mut.0.as_slice_of_cells();
    /// v_mut[2].set(4);
    /// assert_eq!(v, [1, 2, 4]);
    pub fn wrap_mut(r: &'a mut T) -> Self {
        Self {
            inner: (Mut(Cell::from_mut(r)),),
            phantom: PhantomData,
        }
    }
}

impl<'a, T> LifeRef<'a, T> {
    /// Wraps inner value into 1-tuple for chaining with other methods.
    /// # Example
    /// ```
    /// let u = borrow_as::LifeRef::from(true);
    /// assert!(*u);
    /// let uu = u.wrap_life();
    /// assert!(uu.0);
    pub fn wrap_life(self) -> LifeRef<'a, (T,)> {
        LifeRef {
            inner: (self.inner,),
            phantom: PhantomData,
        }
    }

    /// Extends inner tuple by one element which represents passed immutable reference. Supports extending up to 16 elements.
    /// # Example
    /// ```
    /// let t = (0, 1);
    /// let s = String::from("Referenced");
    /// let r = borrow_as::LifeRef::wrap_ref(&t).add_ref(&s[..3]).add_ref(&42);
    /// assert_eq!(r.0, &t);
    /// assert_eq!(r.1, "Ref");
    /// assert_eq!(r.2, &42);
    pub fn add_ref<U>(self, r: &'a U) -> LifeRef<'a, T::Output> where
    T: Append<Ref<U>>,
    U: 'a + ?Sized {
        let t = self.inner;
        let v = t.append(Ref(r));
        LifeRef {
            inner: v,
            phantom: PhantomData,
        }
    }

    /// Extends inner tuple by one element which represents passed mutable reference. Supports extending up to 16 elements.
    /// # Example
    /// ```
    /// let mut t = (0, 1);
    /// let mut s = String::from("Unaltered");
    /// let r = borrow_as::LifeRef::wrap_mut(&mut t).add_mut(&mut s);
    ///
    /// let (y, x) = r.0.get();
    /// r.0.set((x, y));
    ///
    /// let mut r1 = r.1.take();
    /// r1.replace_range(..3, "A");
    /// r.1.set(r1);
    ///
    /// assert_eq!(t, (1, 0));
    /// assert_eq!(s, "Altered");
    pub fn add_mut<U>(self, r: &'a mut U) -> LifeRef<'a, T::Output> where
    T: Append<Mut<U>>,
    U: 'a + ?Sized {
        let t = self.inner;
        let v = t.append(Mut(Cell::from_mut(r)));
        LifeRef {
            inner: v,
            phantom: PhantomData,
        }
    }

    /// Extends inner tuple with extracted value from another `LifeRef`.
    ///
    /// Note: `other` can't outlive `self` and its lifetime will be shortened accordingly.
    /// # Example
    /// ```
    /// use borrow_as::LifeRef as Life;
    /// struct A;
    /// struct B;
    /// struct C;
    /// let a = Life::from(A).wrap_life();
    /// let b = Life::from(B);
    /// let c = Life::from(C);
    /// let ab = a.add_life(b).wrap_life();
    /// let abc: Life<'_, ((A, B), C)> = ab.add_life(c);
    pub fn add_life<'b, U>(self, other: LifeRef<'b, U>) -> LifeRef<'a, T::Output> where
    T: Append<U>,
    'b: 'a {
        let t = self.inner;
        let v = t.append(other.inner);
        LifeRef {
            inner: v,
            phantom: PhantomData,
        }
    }
    
    /// Converts wrapped value from one type to another.
    /// # Example
    /// ```
    /// use borrow_as::*;
    /// struct Test {
    ///     x: Mut<String>,
    ///     y: Ref<i32>,
    ///     z: Mut<bool>,
    /// }
    /// 
    /// let int = 42;
    /// let mut string = String::from("Unaltered");
    /// let mut flag = false;
    /// let test = LifeRef::
    ///     wrap_mut(&mut string)
    ///     .add_ref(&int)
    ///     .add_mut(&mut flag)
    ///     .map_life(|(x, y, z)| Test { x, y, z });
    /// 
    /// let mut x = test.x.take();
    /// x.replace_range(..3, "A");
    /// test.x.set(x);
    /// assert_eq!(test.y, &42);
    /// test.z.set(!test.z.get());
    /// 
    /// assert_eq!(string, "Altered");
    /// assert!(flag);
    pub fn map_life<U>(self, f: impl Fn(T) -> U) -> LifeRef<'a, U> {
        LifeRef {
            inner: f(self.inner),
            phantom: PhantomData,
        }
    }
}

impl<T> From<T> for LifeRef<'_, T> {
    fn from(t: T) -> Self {
        Self {
            inner: t,
            phantom: PhantomData,
        }
    }
}

impl<'a, T> Deref for LifeRef<'a, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<T> Borrow<T> for LifeRef<'_, T> {
    #[inline(always)]
    fn borrow(&self) -> &T {
        self
    }
}

/// Immutable reference.
#[derive(Debug)]
#[repr(transparent)]
pub struct Ref<T: ?Sized>(*const T);

impl<T: ?Sized, U: ?Sized> PartialEq<U> for Ref<T> where for<'a> &'a T: PartialEq<U> {
    #[inline(always)]
    fn eq(&self, other: &U) -> bool {
        self.deref().eq(other)
    }
}

impl<T: ?Sized> Eq for Ref<T> where for<'a> &'a T: Eq + PartialEq<Self> {}

impl<T: ?Sized, U: ?Sized> PartialOrd<U> for Ref<T> where for<'a> &'a T: PartialOrd<U> {
    #[inline(always)]
    fn partial_cmp(&self, other: &U) -> Option<core::cmp::Ordering> {
        self.deref().partial_cmp(other)
    }
}

impl<T: ?Sized> Ord for Ref<T> where for<'a> &'a T: Ord + Eq + PartialOrd<Self> {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.deref().cmp(&other.deref())
    }
}

impl<T: ?Sized> Hash for Ref<T> where for<'a> &'a T: Hash {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.deref().hash(state);
    }
}

impl<T: ?Sized> fmt::Display for Ref<T> where for<'a> &'a T: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t = self.deref();
        if f.alternate() {
            f.debug_tuple("Ref")
            .field(&t)
            .finish()
        }
        else {
            write!(f, "Ref {:?}", &t)
        }
    }
}

impl<T: ?Sized> Deref for Ref<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { &*self.0 }
    }
}

impl<T: ?Sized> AsRef<T> for Ref<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T: ?Sized> Borrow<T> for Ref<T> {
    fn borrow(&self) -> &T {
        self
    }
}

/// Mutable reference via Cell.
#[derive(Debug)]
#[repr(transparent)]
pub struct Mut<T: ?Sized>(*const Cell<T>);

impl<T: ?Sized> Mut<T> {
    unsafe fn get(&self) -> &T {
        (&mut *(self.0 as *mut Cell<T>)).get_mut()
    }
}

impl<T: ?Sized, U: ?Sized> PartialEq<U> for Mut<T> where for<'a> &'a T: PartialEq<U> {
    #[inline(always)]
    fn eq(&self, other: &U) -> bool {
        unsafe { self.get().eq(other) }
    }
}

impl<T: ?Sized> Eq for Mut<T> where for<'a> &'a T: Eq + PartialEq<Self> {}

impl<T: ?Sized, U: ?Sized> PartialOrd<U> for Mut<T> where for<'a> &'a T: PartialOrd<U> {
    #[inline(always)]
    fn partial_cmp(&self, other: &U) -> Option<core::cmp::Ordering> {
        unsafe { self.get().partial_cmp(other) }
    }
}

impl<T: ?Sized> Ord for Mut<T> where for<'a> &'a T: Ord + Eq + PartialOrd<Self> {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        unsafe { self.get().cmp(&other.get()) }
    }
}

impl<T: ?Sized> Hash for Mut<T> where for<'a> &'a T: Hash {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe { self.get().hash(state) };
    }
}

impl<T: ?Sized> fmt::Display for Mut<T> where for<'a> &'a T: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t = unsafe { self.get() };
        if f.alternate() {
            f.debug_tuple("Mut")
            .field(&t)
            .finish()
        }
        else {
            write!(f, "Mut {:?}", &t)
        }
    }
}

impl<T: ?Sized> Deref for Mut<T> {
    type Target = Cell<T>;

    #[inline(always)]
    fn deref(&self) -> &Cell<T> {
        unsafe { &*self.0 }
    }
}

impl<T: ?Sized> AsRef<Cell<T>> for Mut<T> {
    fn as_ref(&self) -> &Cell<T> {
        self
    }
}

impl<T: ?Sized> Borrow<Cell<T>> for Mut<T> {
    fn borrow(&self) -> &Cell<T> {
        self
    }
}