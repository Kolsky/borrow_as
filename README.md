# Borrow As
Partial struct borrowing made easy, including splitting borrows.
# Example
```rust
use std::borrow::Borrow;
use borrow_as::*;

struct X {
    s: String,
    v: Vec<u128>,
    i: i8,
    x: u32,
    f: Box<dyn Fn() -> i32>,
}

impl Default for X {
    fn default() -> Self {
        Self {
            s: String::from("No string for you"),
            v: vec![1, 2, 3],
            i: 0,
            x: 9,
            f: Box::new(|| 0),
        }
    }
}

impl X {
    fn construct_a<'a>(s: &'a String,
                       v: &'a Vec<u128>)
                       -> LifeRef<'a, A> {
        LifeRef::
            wrap_ref(s.as_str())
            .add_ref(v.as_slice())
            .map_life(|(s, v)| A { s, v })
    }

    fn construct_b<'a>(i: &'a mut i8,
                       x: u32,
                       f: &'a (dyn Fn() -> i32 + 'static))
                       -> LifeRef<'a, B> {
        LifeRef::
            wrap_mut(i)
            .add_ref(f)
            .map_life(|(i, f)| B { i, x, f })
    }

    pub fn get_a(&self) -> LifeRef<'_, A> {
        let Self { s, v, .. } = self;
        Self::construct_a(s, v)
    }

    pub fn get_b(&mut self) -> LifeRef<'_, B> {
        let Self { i, x, .. } = self;
        Self::construct_b(i, *x, self.f.as_ref())
    }

    pub fn get_ab(&mut self) -> LifeRef<'_, (A, B)> {
        let Self { s, v, i, x, .. } = self;
        let a = Self::construct_a(s, v);
        let b = Self::construct_b(i, *x, self.f.as_ref());
        a.wrap_life().add_life(b)
    }

    pub fn get_c(&mut self) -> LifeRef<'_, C> {
        let Self { f, v, i, .. } = self;
        LifeRef::
            wrap_mut(f)
            .add_mut(v.as_mut_slice())
            .add_ref(i)
            .map_life(|(f, v, i)| C { f, v, i })
    }
}

struct A {
    pub s: Ref<str>,
    pub v: Ref<[u128]>,
}

pub struct B {
    pub i: Mut<i8>,
    pub x: u32,
    pub f: Ref<dyn Fn() -> i32>,
}

pub struct C {
    pub v: Mut<[u128]>,
    pub i: Ref<i8>,
    pub f: Mut<Box<dyn Fn() -> i32>>,
}

let mut x = X::default();

let a = x.get_a();
assert_eq!(a.s, "No string for you");
assert_eq!(a.v, [1, 2, 3]);

let b = x.get_b();
let b: &B = b.borrow();
assert_eq!(b.i.get(), 0);
assert_eq!(b.x, 9);
assert_eq!((b.f)(), 0);

b.i.set(1);
let c = x.get_c();
let c: &C = c.borrow();
assert_eq!(c.i, &1);

c.f.set(Box::new(|| { 8 }));
let v = c.v.as_slice_of_cells();
v[2].set(4);
let ab = x.get_ab();
assert_eq!(ab.0.v, [1, 2, 4]);
assert_eq!((ab.1.f)(), 8);

assert_eq!(x.s, "No string for you");
assert_eq!(x.v, [1, 2, 4]);
assert_eq!(x.i, 1);
assert_eq!(x.x, 9);
assert_eq!((x.f)(), 8);
```