use derive_getters::Dissolve;
use pyo3::PyAny;
use pyo3::prelude::*;
use std::hash::{Hash, Hasher};

#[derive(Dissolve, Clone, Debug)]
pub struct FallibleBorrowed<'a, 'py>(pub Borrowed<'a, 'py, PyAny>);

impl PartialEq for FallibleBorrowed<'_, '_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(other.0).expect("Failed to compare PyObjects")
    }
}

impl Eq for FallibleBorrowed<'_, '_> {}

impl Hash for FallibleBorrowed<'_, '_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash().expect("Failed to hash PyObjects").hash(state)
    }
}

impl PartialOrd for FallibleBorrowed<'_, '_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FallibleBorrowed<'_, '_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .compare(other.0)
            .expect("Failed to compare PyObjects")
    }
}

#[derive(Dissolve, Clone, Debug)]
pub struct FallibleBound<'py>(pub Bound<'py, PyAny>);

impl PartialEq for FallibleBound<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0).expect("Failed to compare PyObjects")
    }
}

impl Eq for FallibleBound<'_> {}

impl Hash for FallibleBound<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash().expect("Failed to hash PyObjects").hash(state)
    }
}

impl PartialOrd for FallibleBound<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FallibleBound<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .compare(&other.0)
            .expect("Failed to compare PyObjects")
    }
}
