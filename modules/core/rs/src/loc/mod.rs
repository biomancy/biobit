pub use contig::Contig;
pub use locus::{AsLocus, Locus};
pub use orientation::Orientation;
pub use per_orientation::PerOrientation;
pub use per_strand::PerStrand;
pub use segment::{AsSegment, Segment};
pub use strand::Strand;

mod contig;
mod locus;
mod orientation;
mod per_orientation;
mod per_strand;
mod segment;
mod strand;
