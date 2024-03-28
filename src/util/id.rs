use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

pub struct Id<T> {
    value: u64,
    _phantom: PhantomData<fn() -> T>,
}

impl<T> Id<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};

        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        let value = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        Id {
            value,
            _phantom: PhantomData,
        }
    }

    #[allow(dead_code)]
    pub fn from(value: u64) -> Self {
        Id {
            value,
            _phantom: PhantomData,
        }
    }
}

impl<T> Hash for Id<T> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.value.hash(state)
    }
}

impl<T> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Id({:?})", self.value)
    }
}

impl<T> fmt::Display for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl<T> Eq for Id<T> {}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_id() {
        struct A();
        struct B();

        let a: Id<A> = Id::new();
        let b: Id<B> = Id::new();

        assert!(a.value < b.value);
    }
}
