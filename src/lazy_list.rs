use std::cmp::Ordering;
use std::fmt::{Debug, Result as FmtResult};
use std::ops::Deref;
use std::{
    cell::RefCell,
    fmt::{Display, Formatter},
    ops::DerefMut,
    rc::Rc,
};

enum LazyListImpl<T> {
    Lazy(Box<dyn Iterator<Item = T>>),
    Eager(Rc<Vec<T>>),
}
pub(crate) struct LazyList<T> {
    list: RefCell<LazyListImpl<T>>,
}

impl<T> LazyList<T> {
    pub(crate) fn is_empty(&self) -> bool {
        self.eager().is_empty()
    }
    pub(crate) fn len(&self) -> usize {
        self.eager().len()
    }
}
impl<T> From<Vec<T>> for LazyList<T> {
    fn from(value: Vec<T>) -> Self {
        Rc::new(value).into()
    }
}
impl<T> From<Rc<Vec<T>>> for LazyList<T> {
    fn from(value: Rc<Vec<T>>) -> Self {
        LazyList {
            list: RefCell::new(LazyListImpl::Eager(value)),
        }
    }
}

impl<T> From<Box<dyn Iterator<Item = T>>> for LazyList<T> {
    fn from(value: Box<dyn Iterator<Item = T>>) -> Self {
        LazyList {
            list: RefCell::new(LazyListImpl::Lazy(value)),
        }
    }
}

impl<T> LazyList<T> {
    fn eager(&self) -> Rc<Vec<T>> {
        let mut list = self.list.borrow_mut();
        match list.deref_mut() {
            LazyListImpl::Eager(vec) => vec.clone(),
            LazyListImpl::Lazy(iter) => {
                let vec: Vec<_> = iter.collect();
                let vec = Rc::new(vec);
                *list = LazyListImpl::Eager(vec.clone());
                vec
            }
        }
    }
}
impl<T> Clone for LazyList<T> {
    fn clone(&self) -> Self {
        self.eager().into()
    }
}

impl<T: Display> Display for LazyList<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "[")?;
        for (i, t) in self.eager().iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            t.fmt(f)?;
        }
        write!(f, "]")
    }
}
struct ListIterator<T> {
    index: usize,
    list: Rc<Vec<T>>,
}
impl<T: Clone> Iterator for ListIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        self.index += 1;
        self.list.get(index).cloned()
    }
}
impl<T> ListIterator<T> {
    fn new(list: Rc<Vec<T>>) -> Self {
        Self { index: 0, list }
    }
}
enum LazyListIteratorImpl<T> {
    Eager(ListIterator<T>),
    Lazy(Box<dyn Iterator<Item = T>>),
}
pub(crate) struct LazyListIterator<T> {
    iter: LazyListIteratorImpl<T>,
}
impl<T: Clone> Iterator for LazyListIteratorImpl<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Eager(e) => e.next(),
            Self::Lazy(l) => l.next(),
        }
    }
}
impl<T: Clone> Iterator for LazyListIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<T: Clone> IntoIterator for LazyListImpl<T> {
    type IntoIter = LazyListIterator<T>;
    type Item = T;
    fn into_iter(self) -> Self::IntoIter {
        let iter = match self {
            LazyListImpl::Eager(e) => LazyListIteratorImpl::Eager(ListIterator::new(e)),
            LazyListImpl::Lazy(l) => LazyListIteratorImpl::Lazy(l),
        };
        LazyListIterator { iter }
    }
}
impl<T: Clone> IntoIterator for LazyList<T> {
    type IntoIter = LazyListIterator<T>;
    type Item = T;
    fn into_iter(self) -> Self::IntoIter {
        self.list.into_inner().into_iter()
    }
}
impl<T: Debug> Debug for LazyList<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(&self.eager(), f)
    }
}

impl<T: PartialEq> PartialEq for LazyList<T> {
    fn eq(&self, other: &Self) -> bool {
        self.eager().deref() == other.eager().deref()
    }
}

impl<T: Eq> Eq for LazyList<T> {}

impl<T: PartialOrd> PartialOrd for LazyList<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.eager().deref().partial_cmp(other.eager().deref())
    }
}
impl<T: Ord> Ord for LazyList<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.eager().deref().cmp(other.eager().deref())
    }
}
