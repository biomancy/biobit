//! Borrow guards and the per-store ledger that prevents aliasing.

mod entry;
mod read;
#[cfg(test)]
mod tests;
mod write;

pub(in crate::access) use entry::Entry;
pub use read::Read;
pub use write::Write;
