use pyo3::PyErr;
use vista::{UnsafeView, UnsafeViewMut, View, ViewBase, ViewMut};

pub trait PyView: ViewBase + View<Error = PyErr> + Send + Sync + 'static {}
impl<T> PyView for T where T: ViewBase + View<Error = PyErr> + Send + Sync + 'static {}

pub trait PyViewMut: ViewBase + ViewMut<Error = PyErr> + Send + Sync + 'static {}
impl<T> PyViewMut for T where T: ViewBase + ViewMut<Error = PyErr> + Send + Sync + 'static {}

pub trait PyViewUnsafe: ViewBase + UnsafeView<Error = PyErr> + Send + Sync + 'static {}
impl<T> PyViewUnsafe for T where T: ViewBase + UnsafeView<Error = PyErr> + Send + Sync + 'static {}

pub trait PyViewMutUnsafe: ViewBase + UnsafeViewMut<Error = PyErr> + Send + Sync + 'static {}
impl<T> PyViewMutUnsafe for T where
    T: ViewBase + UnsafeViewMut<Error = PyErr> + Send + Sync + 'static
{
}
