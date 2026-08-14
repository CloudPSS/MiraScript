use std::f64::consts;

use crate::{MiraAny, MiraContext, MiraError, Result, operations};

use super::{insert_native, number};

pub(super) fn install(context: &mut MiraContext) {
    for (name, value) in [
        ("PI", consts::PI),
        ("E", consts::E),
        ("pi", consts::PI),
        ("e", consts::E),
        ("SQRT1_2", consts::FRAC_1_SQRT_2),
        ("SQRT2", consts::SQRT_2),
        ("LN2", consts::LN_2),
        ("LN10", consts::LN_10),
        ("LOG2E", consts::LOG2_E),
        ("LOG10E", consts::LOG10_E),
    ] {
        context.insert(name, value);
    }

    macro_rules! unary {
        ($name:literal, $operation:expr) => {
            insert_native(context, $name, |_, args| {
                Ok(MiraAny::Number(($operation)(number(args, 0, "x")?)))
            });
        };
    }

    unary!("sign", |value: f64| {
        if value.is_nan() || value == 0.0 {
            value
        } else {
            value.signum()
        }
    });
    unary!("abs", f64::abs);
    unary!("acos", f64::acos);
    unary!("acosh", f64::acosh);
    unary!("asin", f64::asin);
    unary!("asinh", f64::asinh);
    unary!("atan", f64::atan);
    unary!("atanh", f64::atanh);
    unary!("cos", f64::cos);
    unary!("cosh", f64::cosh);
    unary!("sin", f64::sin);
    unary!("sinh", f64::sinh);
    unary!("tan", f64::tan);
    unary!("tanh", f64::tanh);
    unary!("exp", f64::exp);
    unary!("expm1", f64::exp_m1);
    unary!("log", f64::ln);
    unary!("log10", f64::log10);
    unary!("log1p", f64::ln_1p);
    unary!("log2", f64::log2);
    unary!("sqrt", f64::sqrt);
    unary!("cbrt", f64::cbrt);

    insert_native(context, "atan2", |_, args| {
        Ok(MiraAny::Number(
            number(args, 0, "x")?.atan2(number(args, 1, "y")?),
        ))
    });
    insert_native(context, "pow", |_, args| {
        Ok(MiraAny::Number(
            number(args, 0, "x")?.powf(number(args, 1, "y")?),
        ))
    });
    insert_native(context, "random", |call, _| {
        Ok(MiraAny::Number((call.options().providers.random)()))
    });

    for (name, operation) in [
        ("trunc", f64::trunc as fn(f64) -> f64),
        ("floor", f64::floor),
        ("ceil", f64::ceil),
        ("round", round_ties_even),
    ] {
        insert_native(context, name, move |_, args| {
            let value = number(args, 0, "x")?;
            let digits = match args.get(1) {
                None | Some(MiraAny::Nil) => 0,
                Some(value) => {
                    let digits = operations::to_number(value)?;
                    if !digits.is_finite() {
                        return Err(MiraError::runtime("Argument `n` must be a finite integer"));
                    }
                    digits.trunc() as i32
                }
            };
            if !(0..=15).contains(&digits) {
                return Err(MiraError::runtime("Argument `n` must be between 0 and 15"));
            }
            if digits == 0 {
                Ok(MiraAny::Number(operation(value)))
            } else {
                let factor = 10f64.powi(digits);
                Ok(MiraAny::Number(operation(value * factor) / factor))
            }
        });
    }

    insert_native(context, "max", |_, args| {
        let values = numbers(args)?;
        let mut result = f64::NEG_INFINITY;
        for value in values {
            if value.is_nan() {
                return Ok(MiraAny::Number(f64::NAN));
            }
            if value > result || (value == 0.0 && result == 0.0 && value.is_sign_positive()) {
                result = value;
            }
        }
        Ok(MiraAny::Number(result))
    });
    insert_native(context, "min", |_, args| {
        let values = numbers(args)?;
        let mut result = f64::INFINITY;
        for value in values {
            if value.is_nan() {
                return Ok(MiraAny::Number(f64::NAN));
            }
            if value < result || (value == 0.0 && result == 0.0 && value.is_sign_negative()) {
                result = value;
            }
        }
        Ok(MiraAny::Number(result))
    });
    insert_native(context, "hypot", |_, args| {
        let values = numbers(args)?;
        if values.iter().any(|value| value.is_infinite()) {
            return Ok(MiraAny::Number(f64::INFINITY));
        }
        if values.iter().any(|value| value.is_nan()) {
            return Ok(MiraAny::Number(f64::NAN));
        }
        Ok(MiraAny::Number(
            values.into_iter().fold(0.0_f64, f64::hypot),
        ))
    });
    insert_native(context, "sum", |_, args| {
        let mut total = -0.0;
        for value in numbers(args)? {
            total += value;
        }
        Ok(MiraAny::Number(total))
    });
    insert_native(context, "product", |_, args| {
        Ok(MiraAny::Number(numbers(args)?.into_iter().product()))
    });

    insert_native(context, "gamma", |_, args| {
        Ok(MiraAny::Number(gamma(number(args, 0, "x")?)))
    });
    insert_native(context, "factorial", |_, args| {
        let value = number(args, 0, "x")?;
        Ok(MiraAny::Number(if value.is_nan() || value < 0.0 {
            f64::NAN
        } else {
            gamma(value + 1.0)
        }))
    });

    install_bits(context);
}

fn numbers(args: &[MiraAny]) -> Result<Vec<f64>> {
    let values = if args.len() == 1 && args[0].array_len()?.is_some() {
        operations::materialize_array(&args[0])?
    } else {
        args.to_vec()
    };
    values.iter().map(operations::to_number).collect()
}

fn round_ties_even(value: f64) -> f64 {
    let rounded = value.round();
    if (value - rounded).abs() == 0.5 {
        (value / 2.0).round() * 2.0
    } else {
        rounded
    }
}

fn gamma(value: f64) -> f64 {
    const GMHALF: f64 = 5.524_680_040_776_73;
    let bits = value.to_bits();
    let high = ((bits >> 32) & 0x7fff_ffff) as u32;
    let negative = bits >> 63 != 0;

    if high >= 0x7ff0_0000 {
        return value + f64::INFINITY;
    }
    if high < ((0x3ff - 54) << 20) {
        return 1.0 / value;
    }
    if value == value.floor() {
        if negative {
            return f64::NAN;
        }
        if value <= 171.0 {
            return factorial_integer(value.trunc() as usize - 1);
        }
    }
    if high >= 0x4067_0000 {
        if negative {
            return if value.floor() * 0.5 == (value * 0.5).floor() {
                0.0
            } else {
                -0.0
            };
        }
        return value * f64::from_bits(0x7fe0_0000_0000_0000);
    }

    let abs = value.abs();
    let y = abs + GMHALF;
    let mut dy = if abs > GMHALF {
        (y - abs) - GMHALF
    } else {
        (y - GMHALF) - abs
    };
    let mut z = abs - 0.5;
    let mut result = gamma_rational(abs) * (-y).exp();
    if value < 0.0 {
        result = -consts::PI / (sin_pi(abs) * abs * result);
        dy = -dy;
        z = -z;
    }
    result += (dy * (GMHALF + 0.5) * result) / y;
    let half = y.powf(0.5 * z);
    result * half * half
}

fn factorial_integer(value: usize) -> f64 {
    let mut limbs = vec![1_u32];
    for multiplier in 2..=value as u32 {
        let mut carry = 0_u64;
        for limb in &mut limbs {
            let product = u64::from(*limb) * u64::from(multiplier) + carry;
            *limb = product as u32;
            carry = product >> 32;
        }
        if carry != 0 {
            limbs.push(carry as u32);
        }
    }
    integer_limbs_to_f64(&limbs)
}

fn integer_limbs_to_f64(limbs: &[u32]) -> f64 {
    let bit_len =
        (limbs.len() - 1) * 32 + (u32::BITS - limbs.last().unwrap().leading_zeros()) as usize;
    let shift = bit_len.saturating_sub(53);
    let mut significand = 0_u64;
    for bit in 0..bit_len.min(53) {
        let source = shift + bit;
        if limbs[source / 32] & (1_u32 << (source % 32)) != 0 {
            significand |= 1_u64 << bit;
        }
    }
    if shift > 0 {
        let halfway = shift - 1;
        let halfway_set = limbs[halfway / 32] & (1_u32 << (halfway % 32)) != 0;
        let lower_set = (0..halfway).any(|bit| limbs[bit / 32] & (1_u32 << (bit % 32)) != 0);
        if halfway_set && (lower_set || significand & 1 != 0) {
            significand += 1;
        }
    }
    significand as f64 * 2_f64.powi(shift as i32)
}

fn sin_pi(mut value: f64) -> f64 {
    value *= 0.5;
    value = 2.0 * (value - value.floor());
    let n = ((4.0 * value).trunc() + 1.0).div_euclid(2.0).trunc() as i32;
    value = (value - n as f64 * 0.5) * consts::PI;
    match n {
        1 => value.cos(),
        2 => (-value).sin(),
        3 => -value.cos(),
        _ => value.sin(),
    }
}

fn gamma_rational(value: f64) -> f64 {
    const NUMERATOR: [f64; 13] = [
        23_531_376_880.410_76,
        42_919_803_642.649_1,
        35_711_959_237.355_67,
        17_921_034_426.037_21,
        6_039_542_586.352_028,
        1_439_720_407.311_721_6,
        248_874_557.862_054_17,
        31_426_415.585_400_194,
        2_876_370.628_935_372_5,
        186_056.265_395_223_5,
        8_071.672_002_365_816,
        210.824_277_751_579_36,
        2.506_628_274_631_000_2,
    ];
    const DENOMINATOR: [f64; 13] = [
        0.0,
        39_916_800.0,
        120_543_840.0,
        150_917_976.0,
        105_258_076.0,
        45_995_730.0,
        13_339_535.0,
        2_637_558.0,
        357_423.0,
        32_670.0,
        1_925.0,
        66.0,
        1.0,
    ];
    let mut numerator = 0.0;
    let mut denominator = 0.0;
    if value < 8.0 {
        for index in (0..=12).rev() {
            numerator = numerator * value + NUMERATOR[index];
            denominator = denominator * value + DENOMINATOR[index];
        }
    } else {
        for index in 0..=12 {
            numerator = numerator / value + NUMERATOR[index];
            denominator = denominator / value + DENOMINATOR[index];
        }
    }
    numerator / denominator
}

fn to_u32(value: f64) -> u32 {
    if !value.is_finite() || value == 0.0 {
        return 0;
    }
    value.trunc().rem_euclid(4_294_967_296.0) as u32
}

fn install_bits(context: &mut MiraContext) {
    macro_rules! binary {
        ($name:literal, $operation:expr) => {
            insert_native(context, $name, |_, args| {
                let left = to_u32(number(args, 0, "x")?);
                let right = to_u32(number(args, 1, "y")?);
                Ok(MiraAny::Number(($operation)(left, right) as f64))
            });
        };
    }
    binary!("b_and", |a: u32, b: u32| (a & b) as i32);
    binary!("b_or", |a: u32, b: u32| (a | b) as i32);
    binary!("b_xor", |a: u32, b: u32| (a ^ b) as i32);
    binary!("shl", |a: u32, b: u32| a.wrapping_shl(b & 31) as i32);
    binary!("sar", |a: u32, b: u32| (a as i32) >> (b & 31));
    binary!("shr", |a: u32, b: u32| a >> (b & 31));
    insert_native(context, "b_not", |_, args| {
        Ok(MiraAny::Number(
            (!to_u32(number(args, 0, "x")?) as i32) as f64,
        ))
    });
}
