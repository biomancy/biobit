pub use contig::Contig;
pub use interval::{Interval, LikeInterval};
pub use locus::{LikeLocus, Locus};
pub use orientation::Orientation;
pub use strand::Strand;

mod strand;
mod orientation;
mod locus;
mod contig;
mod interval;
