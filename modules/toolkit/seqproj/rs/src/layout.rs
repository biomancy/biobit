use std::path::PathBuf;

pub use biobit_core_rs::ngs;

// Inspired by Salmon: https://salmon.readthedocs.io/en/latest/library_type.html
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Layout {
    Single {
        file: PathBuf,
    },
    Paired {
        orientation: Option<ngs::MatesOrientation>,
        files: (PathBuf, PathBuf),
    },
}

// impl Display for SeqLib {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         match self {
//             SeqLib::Single { strandedness } => write!(f, "S{}", strandedness),
//             SeqLib::Paired {
//                 strandedness,
//                 orientation,
//             } => write!(f, "P{}{}", strandedness, orientation),
//         }
//     }
// }
