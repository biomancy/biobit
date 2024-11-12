use derive_getters::Dissolve;
use pyo3::prelude::*;
use pyo3::PyAny;
use std::hash::{Hash, Hasher};

#[derive(Dissolve, Clone, Debug)]
pub struct FallibleBorrowed<'a, 'py>(pub Borrowed<'a, 'py, PyAny>);

impl<'a, 'py> PartialEq for FallibleBorrowed<'a, 'py> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(other.0).expect("Failed to compare PyObjects")
    }
}

impl<'a, 'py> Eq for FallibleBorrowed<'a, 'py> {}

impl<'a, 'py> Hash for FallibleBorrowed<'a, 'py> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash().expect("Failed to hash PyObjects").hash(state)
    }
}

impl<'a, 'py> PartialOrd for FallibleBorrowed<'a, 'py> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, 'py> Ord for FallibleBorrowed<'a, 'py> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .compare(other.0)
            .expect("Failed to compare PyObjects")
    }
}

#[derive(Dissolve, Clone, Debug)]
pub struct FallibleBound<'py>(pub Bound<'py, PyAny>);

impl<'py> PartialEq for FallibleBound<'py> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0).expect("Failed to compare PyObjects")
    }
}

impl<'py> Eq for FallibleBound<'py> {}

impl<'py> Hash for FallibleBound<'py> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash().expect("Failed to hash PyObjects").hash(state)
    }
}

impl<'py> PartialOrd for FallibleBound<'py> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'py> Ord for FallibleBound<'py> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .compare(&other.0)
            .expect("Failed to compare PyObjects")
    }
}
