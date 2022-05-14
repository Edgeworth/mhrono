use derive_more::Display;

/// Number of occurrences of something.
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Display, Ord, PartialOrd)]
pub struct Cycle(pub i64);
