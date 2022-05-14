use derive_more::Display;

use crate::duration::Duration;

/// Number of times something happens in a second. Hertz.
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Display, Ord, PartialOrd)]
pub struct Freq {
    dur: Duration,
}
