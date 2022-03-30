use std::fmt;
use std::iter;
use std::slice;
use std::vec;

/// Enum type which stores either no items, a single item or a vector of items.
/// Allows you to reduce heap allocations in cases where you mostly have collections which are empty
/// or consist only of a single item.
pub enum Plural<T> {
    None,
    One(T),
    Many(Vec<T>),
}

impl<T> Plural<T> {
    pub fn iter(&self) -> Iter<T> {
        match self {
            Plural::None => Iter::None,
            Plural::One(val) => Iter::One(iter::once(val)),
            Plural::Many(vals) => Iter::Many(vals.iter()),
        }
    }
}

impl<T> IntoIterator for Plural<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Plural::None => IntoIter::None,
            Plural::One(val) => IntoIter::One(iter::once(val)),
            Plural::Many(vals) => IntoIter::Many(vals.into_iter()),
        }
    }
}

impl<T> Clone for Plural<T> where T: Clone {
    fn clone(&self) -> Self {
        match self {
            Self::None => Self::None,
            Self::One(val) => Self::One(val.clone()),
            Self::Many(vals) => Self::Many(vals.clone()),
        }
    }
}

impl<T> fmt::Debug for Plural<T> where T: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::One(val) => write!(f, "One({:?})", val),
            Self::Many(vals) => write!(f, "Many({:?})", vals),
        }
    }
}

impl<'a, T> IntoIterator for &'a Plural<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub enum Iter<'a, T> {
    None,
    One(iter::Once<&'a T>),
    Many(slice::Iter<'a, T>),
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::None => None,
            Self::One(it) => it.next(),
            Self::Many(it) => it.next(),
        }
    }
}

pub enum IntoIter<T> {
    None,
    One(iter::Once<T>),
    Many(vec::IntoIter<T>),
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::None => None,
            Self::One(it) => it.next(),
            Self::Many(it) => it.next(),
        }
    }    
}
