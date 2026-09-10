use super::{C, INF, NAN, radial};

pub(super) fn add((a, b): C, (c, d): C) -> C {
    (a + c, b + d)
}
pub(super) fn subtract((a, b): C, (c, d): C) -> C {
    (a - c, b - d)
}
pub(super) fn neg((a, b): C) -> C {
    (-a, -b)
}
pub(super) fn conj((a, b): C) -> C {
    (a, -b)
}
pub(super) fn multiply((a, b): C, (c, d): C) -> C {
    if b == 0.0 && d == 0.0 {
        return (a * c, radial(a, d) + radial(c, b));
    }
    if [a, b, c, d].iter().all(|x| x.is_finite())
        && [a * c, b * d, a * d, b * c].iter().any(|x| !x.is_finite())
    {
        let s = a.abs().max(b.abs());
        let t = c.abs().max(d.abs());
        return (
            ((a / s * (c / t) - b / s * (d / t)) * s.min(t)) * s.max(t),
            ((a / s * (d / t) + b / s * (c / t)) * s.min(t)) * s.max(t),
        );
    }
    (a * c - b * d, a * d + b * c)
}
fn sum_over(x: f64, y: f64, q: f64) -> f64 {
    let sum = x + y;
    if !sum.is_finite() && x.is_finite() && y.is_finite() {
        x / q + y / q
    } else {
        sum / q
    }
}
pub(super) fn divide((a, b): C, (mut c, mut d): C) -> C {
    // f64::max ignores a NaN, unlike Math.max.
    if c.is_nan() || d.is_nan() {
        return (NAN, NAN);
    }
    let s = c.abs().max(d.abs());
    if s == 0.0 {
        return (INF.copysign(c) * a, INF.copysign(c) * b);
    }
    if s == INF && a.is_finite() && b.is_finite() {
        c = (if c.abs() == INF { 1.0_f64 } else { 0.0 }).copysign(c);
        d = (if d.abs() == INF { 1.0_f64 } else { 0.0 }).copysign(d);
        return (0.0 * (a * c + b * d), 0.0 * (b * c - a * d));
    }
    c /= s;
    d /= s;
    let q = c * c + d * d;
    if (!(a / s).is_finite() || !(b / s).is_finite()) && a.is_finite() && b.is_finite() {
        let u = a.abs().max(b.abs());
        return (
            ((a / u * c + b / u * d) / q * u) / s,
            ((b / u * c - a / u * d) / q * u) / s,
        );
    }
    (
        sum_over(a / s * c, b / s * d, q),
        sum_over(b / s * c, -a / s * d, q),
    )
}
pub(super) fn polar(r: f64, theta: f64) -> C {
    (radial(r, theta.cos()), radial(r, theta.sin()))
}
