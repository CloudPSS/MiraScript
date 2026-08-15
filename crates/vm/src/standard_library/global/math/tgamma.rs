use std::f64::consts;

use crate::standard_library::{insert_native, number};
use crate::{MiraAny, MiraContext};

pub(super) fn install(context: &mut MiraContext) {
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
