/// Traits to mark ds supporting multiple read-only or mutable views to the underlying data.
///
/// These traits return a simple facade, and the actual views are constructed by calling
/// facade methods. Introducing a facade might seem redundant, but it is necessary to ensure a
/// consistent way of accessing all views supported by a given collection without requiring users
/// to type out the full view type.


pub trait View {
    type Output<'a>
        where Self: 'a;

    /// Returns a facade that can be used to create read-only views of the collection.
    fn view(&self) -> Self::Output<'_>;
}

pub trait ViewMut {
    type Output<'a>
        where Self: 'a;

    /// Returns a facade that can be used to create mutable views of the collection.
    fn view_mut(&mut self) -> Self::Output<'_>;
}
