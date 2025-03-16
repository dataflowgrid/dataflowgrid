/* This file is part of dataFlowGrid. See file LICENSE for full license details. (c) 2025 Alexander Zich */

use std::ops::Deref;

enum Callback<T> {
    None,
    Function(String, Box<dyn Fn(T)>)
}
use std::fmt::{Debug, Formatter, Result as FmtResult};

impl<T> Debug for Callback<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Callback::None => write!(f, "None"),
            Callback::Function(name, _) => {
                write!(f, "Function {name}({name})")
            }
        }
    }
}

#[derive(Debug)]
pub struct Reusable<T> {
    reusable_inner: *mut ReusableInner<T>
}

pub struct ReusableInner<T> {
    inner: T,
    strong_count: usize,
    weak_count: usize,
    callback: Callback<T>,
}

impl<T> Reusable<T> {
    pub fn new(inner: T, callback: Callback<T>) -> Self {
        let inner = Box::new(ReusableInner {
            inner,
            strong_count: 1,
            weak_count: 0,
            callback,
        });

        Reusable {
            reusable_inner: Box::into_raw(inner)
        }
    }
}

impl<T> Clone for Reusable<T> {
    fn clone(&self) -> Self {
        unsafe {self.reusable_inner.as_mut()}.unwrap().strong_count += 1;
        Reusable {
            reusable_inner: self.reusable_inner
        }
    }
}

impl<T> Deref for Reusable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.reusable_inner).inner }
    }
}

impl<T> Drop for Reusable<T> {
    fn drop(&mut self) {
        let mut inner = unsafe {Box::from_raw(self.reusable_inner)};
        inner.strong_count -= 1;
        if inner.strong_count == 0 {
            match inner.callback {
                Callback::None => {}
                Callback::Function(_, ref callback) => (callback)(inner.inner)
            }
        } else {
            self.reusable_inner = Box::into_raw(inner);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn test_reusable() {
        let inner = Rc::new(RefCell::new(0));
        let callback = Callback::Function(String::from("test"), Box::new(|x: Rc<RefCell<i32>>| {
            *x.borrow_mut() += 1;
        }));
        println!("{:?}", callback);
        let reusable = Reusable::new(inner.clone(), callback);
        assert_eq!(*reusable.borrow(), 0);
        assert_eq!(Rc::strong_count(&inner), 2);
        let cloned = reusable.clone();
        assert_eq!(*reusable.borrow(), 0);
        assert_eq!(*cloned.borrow(), 0);
        drop(reusable);
        assert_eq!(*cloned.borrow(), 0);
        assert_eq!(Rc::strong_count(&inner), 2);
        drop(cloned);
        //after the second drop the callback should have been called and RC should be decreased
        assert_eq!(*inner.borrow(), 1);
        assert_eq!(Rc::strong_count(&inner), 1);
    }
}