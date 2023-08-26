/// Returns |a| if |b| is not comparable.
pub fn pmin<X: PartialOrd + Clone>(a: &X, b: &X) -> X {
    if b < a { b.clone() } else { a.clone() }
}

/// Returns |a| if |b| is not comparable.
pub fn pmax<X: PartialOrd + Clone>(a: &X, b: &X) -> X {
    if b > a { b.clone() } else { a.clone() }
}
