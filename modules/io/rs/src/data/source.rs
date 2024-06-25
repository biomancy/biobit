use std::fmt::Display;

pub trait Batch {
    type Item;

}

// Simplicity is the best thing ever. I don't need to overcomplicate things. I can just use the built-in iterator methods.
// And I don't really need any sort of abstraction over data sources.
// Everything will accept just an iterator over alignment segments. That's it. Nothing more, nothing less.
// What I might want to do is to create a trait that will allow me to fetch alignment segments from a data source.
// Do I plan to work with anything but BAM?

// I do need some abstractions.
// First of all - to build a nested processing pipeline and decorate it with some additional functionality.
// Second - to abstract over different data sources. I might want to work with BAM, CRAM, or even a custom data source.
// Third - to provide a common interface for data sources that can be used in a data flow.

// Ok, so the first thing to have is a trait for a data source that will return a boxed iterator (same for indexed stuff).
// And a boxed iterator will hold


// Do I even need anything since I already have a nice and clean iterator interface? Not really?...



// Data source -> thing that can be read from

// Data flow -> composable flow of data from a data source. Do I even need one? Like, I can just use the built-in iterator methods.

// Do I truly need a Source trait? What is the purpose of it?
// * To abstract over different data sources?
// * To provide a common interface for data sources?
// * To provide a common interface for data sources that can be used in a data flow?
// Not sure if I need it. I can just use the built-in iterator methods.

// I want flows as a completely separate thing. I want to be able to build a flow and then run it given a data source.
// Then the result of the flow would be an iterator over batches of data items. And I can just use the built-in iterator methods to process them.
// Sounds suspiciously like Resoultion kind of things.



// Batch -> a collection of data items



// pub trait Source {
//     type Item;
//
//     type Iter<'a>: Iterator<Item=AlignmentSegments<Self::Idx>> + 'a
//     where
//         Self: 'a;
//
//     /// Fetch reads from a specific region of the reference genome.
//     /// * `contig` - The contig to fetch reads from.
//     /// * `start` - The start position of the region.
//     /// * `end` - The end position of the region.
//     /// * Returns an iterator of alignment blocks in the region.
//     fn fetch(&mut self, contig: &str, start: Self::Idx, end: Self::Idx) -> Self::Iter<'_>;
//
//     /// Return statistics about processed alignemnts.
//     /// * Returns an object containing statistics about the source. The content is implementation-specific.
//     fn stats(&self) -> Self::Stats;
// }
//
// pub trait Batch {}
