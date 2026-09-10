//! Cartesian kernels mirror the TypeScript implementation; only the VM boundary
//! performs conversion. No host complex value escapes into MiraScript.
use std::f64::consts::LN_2;

use crate::standard_library::required;
use crate::{MiraValue, Result, Runtime, operations};

pub(super) type C = (f64, f64);
const INF: f64 = f64::INFINITY;
const NAN: f64 = f64::NAN;

pub(super) fn parse(
    runtime: &mut Runtime,
    args: &[MiraValue],
    index: usize,
    name: &'static str,
) -> Result<C> {
    let value = *required(args, index, name)?;
    if let Some(handle) = value.as_record() {
        // Check both keys before materializing either field of a host record.
        let record = runtime.get_record_dyn(handle)?;
        if record.index_of("0").is_some() && record.index_of("1").is_some() {
            let re = operations::record_get(runtime, value, "0")?.unwrap_or(MiraValue::NIL);
            let im = operations::record_get(runtime, value, "1")?.unwrap_or(MiraValue::NIL);
            return Ok((
                operations::to_number(runtime, re)?,
                operations::to_number(runtime, im)?,
            ));
        }
    }
    Ok((operations::to_number(runtime, value)?, 0.0))
}

pub(super) fn unary(runtime: &mut Runtime, args: &[MiraValue], f: fn(C) -> C) -> Result<MiraValue> {
    let value = parse(runtime, args, 0, "z")?;
    runtime.insert(f(value))
}
pub(super) fn binary(
    runtime: &mut Runtime,
    args: &[MiraValue],
    f: fn(C, C) -> C,
) -> Result<MiraValue> {
    let a = parse(runtime, args, 0, "a")?;
    let b = parse(runtime, args, 1, "b")?;
    runtime.insert(f(a, b))
}

fn radial(r: f64, x: f64) -> f64 {
    if x == 0.0 && !r.is_nan() {
        0.0_f64.copysign(r) * x
    } else {
        r * x
    }
}
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
fn log_abs((x, y): C) -> f64 {
    if x.is_nan() || y.is_nan() {
        return NAN;
    }
    let a = x.abs().max(y.abs());
    let b = x.abs().min(y.abs());
    if a == INF {
        return INF;
    }
    if a == 0.0 {
        return -INF;
    }
    a.ln() + 0.5 * ((b / a).powi(2)).ln_1p()
}
pub(super) fn log(z: C) -> C {
    (log_abs(z), z.1.atan2(z.0))
}
pub(super) fn log_base(z: C, base: f64) -> C {
    let (a, b) = log(z);
    (a / base, b / base)
}
pub(super) fn sqrt((x, y): C) -> C {
    if y.abs() == INF {
        return (INF, y);
    }
    if x == INF {
        return (INF, if y.is_nan() { NAN } else { 0.0_f64.copysign(y) });
    }
    if x == -INF {
        return (if y.is_nan() { NAN } else { 0.0 }, INF.copysign(y));
    }
    if x.is_nan() || y.is_nan() {
        return (NAN, NAN);
    }
    let s = x.abs().max(y.abs());
    if s == 0.0 {
        return (0.0, y);
    }
    let t = s.sqrt() * (((x / s).hypot(y / s) + x.abs() / s) / 2.0).sqrt();
    if x >= 0.0 {
        (t, y / t / 2.0)
    } else {
        (y.abs() / t / 2.0, t.copysign(y))
    }
}
pub(super) fn exp((x, y): C) -> C {
    polar(x.exp(), y)
}
pub(super) fn expm1((x, y): C) -> C {
    (
        x.exp_m1() * y.cos() - 2.0 * (y / 2.0).sin().powi(2),
        radial(x.exp(), y.sin()),
    )
}
pub(super) fn log1p((x, y): C) -> C {
    let re = if x.abs() < 0.5 && y.abs() < 0.5 {
        0.5 * (x * (2.0 + x) + y * y).ln_1p()
    } else {
        log_abs((1.0 + x, y))
    };
    (re, y.atan2(1.0 + x))
}
pub(super) fn pow(z: C, w: C) -> C {
    if w.0 == 0.0 && w.1 == 0.0 {
        return (1.0, 0.0);
    }
    if w.0 == 1.0 && w.1 == 0.0 {
        return z;
    }
    if z.0 == 0.0 && z.1 == 0.0 {
        if w.1 != 0.0 || w.0.is_nan() {
            return (NAN, NAN);
        }
        return polar(if w.0 > 0.0 { 0.0 } else { INF }, w.0 * z.1.atan2(z.0));
    }
    if w.1 == 0.0 {
        if z.1 == 0.0 && w.0.is_finite() && w.0.trunc() == w.0 {
            return (z.0.powf(w.0), 0.0_f64.copysign(z.1));
        }
        return polar((log_abs(z) * w.0).exp(), z.1.atan2(z.0) * w.0);
    }
    exp(multiply(w, log(z)))
}
pub(super) fn cbrt(z: C) -> C {
    polar((log_abs(z) / 3.0).exp(), z.1.atan2(z.0) / 3.0)
}
pub(super) fn sin((x, y): C) -> C {
    (radial(y.cosh(), x.sin()), radial(y.sinh(), x.cos()))
}
pub(super) fn cos((x, y): C) -> C {
    (radial(y.cosh(), x.cos()), -radial(y.sinh(), x.sin()))
}
pub(super) fn sinh((x, y): C) -> C {
    (radial(x.sinh(), y.cos()), radial(x.cosh(), y.sin()))
}
pub(super) fn cosh((x, y): C) -> C {
    (radial(x.cosh(), y.cos()), radial(x.sinh(), y.sin()))
}
pub(super) fn tan((x, y): C) -> C {
    let t = (-2.0 * y.abs()).exp();
    let denominator = (1.0 - t).powi(2) + 4.0 * t * x.cos().powi(2);
    (
        4.0 * t * x.sin() * x.cos() / denominator,
        (-(-4.0 * y.abs()).exp_m1() / denominator).copysign(y),
    )
}
pub(super) fn tanh((x, y): C) -> C {
    let (a, b) = tan((y, x));
    (b, a)
}
pub(super) fn asin((x, y): C) -> C {
    if y == 0.0 && x.abs() <= 1.0 {
        return (x.asin(), y);
    }
    let s = if x.is_nan() || y.is_nan() {
        NAN
    } else {
        x.abs().max(y.abs())
    };
    if s > 1e150 {
        return (x.atan2(y.abs()), (log_abs((x, y)) + LN_2).copysign(y));
    }
    let (h, im) = inverse_geometry(x, y);
    (x.atan2(h), im)
}
fn inverse_geometry(x: f64, y: f64) -> C {
    let ax = x.abs();
    let r = (ax + 1.0).hypot(y);
    let t = (ax - 1.0).hypot(y);
    let a = r / 2.0 + t / 2.0;
    let delta = if ax <= 1.0 {
        (y / (r + (1.0 + ax)).sqrt()).hypot(y / (t + (1.0 - ax)).sqrt()) / std::f64::consts::SQRT_2
    } else {
        (a - 1.0).sqrt()
    };
    let gap = if ax <= 1.0 {
        (1.0 - ax).sqrt().hypot(delta)
    } else {
        (y / (r + (ax + 1.0)).sqrt()).hypot(y / (t + (ax - 1.0)).sqrt()) / std::f64::consts::SQRT_2
    };
    (
        gap * (a + ax).sqrt(),
        (delta * (delta + (a + 1.0).sqrt())).ln_1p().copysign(y),
    )
}
pub(super) fn acos((x, y): C) -> C {
    if y == 0.0 && x.abs() <= 1.0 {
        return (x.acos(), -y);
    }
    if !x.is_nan() && !y.is_nan() && x.abs().max(y.abs()) > 1e150 {
        return (y.abs().atan2(x), -(log_abs((x, y)) + LN_2).copysign(y));
    }
    let (h, im) = inverse_geometry(x, y);
    (h.atan2(x), -im)
}
pub(super) fn atan((x, y): C) -> C {
    if y == 0.0 {
        return (x.atan(), y);
    }
    let re = (x.atan2(1.0 - y) + x.atan2(1.0 + y)) / 2.0;
    let h = x.hypot(y - 1.0);
    let im = if h == 0.0 {
        INF
    } else if y.abs() < 0.5 * h {
        0.25 * ((4.0 * y / h) / h).ln_1p()
    } else {
        (log_abs((x, y + 1.0)) - log_abs((x, y - 1.0))) / 2.0
    };
    (re, im)
}
pub(super) fn asinh((x, y): C) -> C {
    let (a, b) = asin((-y, x));
    (b, -a)
}
pub(super) fn acosh(z: C) -> C {
    let (a, b) = acos(z);
    (b.abs(), a.copysign(z.1))
}
pub(super) fn atanh((x, y): C) -> C {
    let (a, b) = atan((-y, x));
    (b, -a)
}
