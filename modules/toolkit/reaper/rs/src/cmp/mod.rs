// In the future we might have a proper trait for a "cmp" functionality.
// E.g. cmp.Poisson, cmp.Difference, cmp.Binomial and so on.
// For now, it's just the enrichment, and it's better to keep things simple.

pub use enrichment::Enrichment;

mod enrichment;
