// Format specification: https://samtools.github.io/hts-specs/BEDv1.pdf

// Mandatory fields:
// 1. seqid: [[:alnum:]_]{1,255}
// 2. start: u64
// 3. end: u64
// 4. name: [\x20-\x7e]{1,255}
// 5. score: u16 [0, 1000]
// 6. orientation: [+|-|.]
// 7. thickStart: u64
// 8. thickEnd: u64
// 9. itemRgb: ([0, 255], [0, 255], [0, 255]) | 0
// 10. blockCount: u64 [1, chromEnd − chromStart]
// 11. blockSizes: vec[u64]
// 12. blockStarts: vec[u64]

// All blockSizes must be > 0
// All blockStarts must be in ascending order, in coordinates relative to chromStart
// For all i, blockStarts[i] + blockSizes[i] must be less than or equal to blockStarts[i + 1]
// For all i, start + blockStarts[i] + blockSizes[i] must be >= start and <= end
// blockStarts[0] must be equal to 0
// start + blockStarts[blockCount – 1] + blockSizes[blockCount – 1] must be equal to end

mod reader;
mod record;
pub mod validate;
mod writer;

pub use reader::Reader;

pub use record::{
    Bed12, Bed12MutOp, Bed12Op, Bed3, Bed3MutOp, Bed3Op, Bed4, Bed4MutOp, Bed4Op, Bed5, Bed5MutOp,
    Bed5Op, Bed6, Bed6MutOp, Bed6Op, Bed8, Bed8MutOp, Bed8Op, Bed9, Bed9MutOp, Bed9Op,
};

pub use writer::Writer;
