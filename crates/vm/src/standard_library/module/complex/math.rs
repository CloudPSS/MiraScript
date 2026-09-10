use std::f64::consts::LN_2;

use super::{C, INF, NAN, multiply, polar, radial};

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
