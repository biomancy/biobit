//! Locking storage backends and their allocation-owning leases.

mod rwlock;
#[cfg(test)]
mod tests;

pub use rwlock::ArcRwLock;
