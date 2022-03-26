use std::ops::Sub;

use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Copy,
    Clone,
    Display,
    Serialize,
    Deserialize,
)]
#[display(fmt = "[{}, {})", st, en)]
pub struct Span<T: PartialOrd + Copy> {
    pub st: T,
    pub en: T, // Exclusive.
}

/// Returns |a| if |b| is not comparable.
pub fn pmin<X: PartialOrd + Copy>(a: X, b: X) -> X {
    if b < a { b } else { a }
}

/// Returns |a| if |b| is not comparable.
pub fn pmax<X: PartialOrd + Copy>(a: X, b: X) -> X {
    if b > a { b } else { a }
}

impl<T: PartialOrd + Copy> Span<T> {
    pub fn new(st: impl Into<T>, en: impl Into<T>) -> Self {
        Self { st: st.into(), en: en.into() }
    }

    pub fn cover(a: Self, b: Self) -> Self {
        if a.is_empty() {
            b
        } else if b.is_empty() {
            a
        } else {
            Span::new(pmin(a.st, b.st), pmax(a.en, b.en))
        }
    }

    pub fn contains(&self, t: T) -> bool {
        t >= self.st && t < self.en
    }

    pub fn contains_span(&self, s: &Self) -> bool {
        self.st <= s.st && self.en >= s.en
    }

    pub fn is_empty(&self) -> bool {
        self.st == self.en
    }

    pub fn intersect(&self, s: &Self) -> Option<Self> {
        let st = pmax(self.st, s.st);
        let en = pmin(self.en, s.en);
        if en > st { Some(Span::new(st, en)) } else { None }
    }
}

impl<T: PartialOrd + Copy + Sub> Span<T> {
    pub fn size(&self) -> <T as Sub>::Output {
        self.en - self.st
    }
}
