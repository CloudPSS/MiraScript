//! Time-related global functions.

use indexmap::IndexMap;

use crate::{MiraError, MiraValue, Result, Runtime, RuntimeErrorKind, operations};

use crate::standard_library::insert_native;

pub(super) fn install(context: &mut Runtime) {
    insert_native(context, "to_timestamp", |call, args| {
        match timestamp(call, args.first()) {
            Ok(value) => Ok(MiraValue::number(value as f64)),
            Err(_) if args.len() > 1 => Ok(args[1]),
            Err(error) => Err(error),
        }
    });
    insert_native(context, "to_datetime", |call, args| {
        let fallback = args.get(2);
        let timestamp = match timestamp(call, args.first()) {
            Ok(value) => value,
            Err(_) if fallback.is_some() => return Ok(*fallback.unwrap()),
            Err(error) => return Err(error),
        };
        let offset = match args.get(1) {
            None => 0.0,
            Some(value) if value.is_nil() => 0.0,
            Some(value) => operations::to_number(call, *value)?,
        };
        if !offset.is_finite() || !(-24.0..=24.0).contains(&offset) {
            return Err(MiraError::runtime(RuntimeErrorKind::TimeOffsetOutOfRange));
        }
        call.insert(datetime_record(timestamp, offset))
    });
    insert_native(context, "to_iso8601", |call, args| {
        match timestamp(call, args.first()) {
            Ok(value) => call.insert(iso8601(value)),
            Err(_) if args.len() > 1 => Ok(args[1]),
            Err(error) => Err(error),
        }
    });
}

fn timestamp(call: &mut crate::Runtime, value: Option<&MiraValue>) -> Result<i64> {
    match value {
        None => Ok(call.options().providers.now_millis()),
        Some(value) if value.is_nil() => Ok(call.options().providers.now_millis()),
        Some(value)
            if value
                .as_number()
                .is_some_and(|number| number.is_finite() && number.abs() <= 8.64e15) =>
        {
            Ok(value.as_number().expect("checked number").trunc() as i64)
        }
        Some(value) if value.is_string() => {
            if let Ok(number) = operations::to_number(call, *value)
                && number.is_finite()
                && number.abs() <= 8.64e15
            {
                return Ok(number.trunc() as i64);
            }
            let source = value.as_str(call)?.expect("matched string");
            parse_iso8601(source)
                .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::InvalidDateTime))
        }
        Some(value) => Err(MiraError::runtime(RuntimeErrorKind::InvalidTimestamp {
            actual: value.value_type(),
        })),
    }
}

fn datetime_record(timestamp: i64, offset: f64) -> IndexMap<String, MiraValue> {
    let adjusted = timestamp as i128 + (offset * 3_600_000.0).trunc() as i128;
    let days = adjusted.div_euclid(86_400_000);
    let day_millis = adjusted.rem_euclid(86_400_000) as i64;
    let (year, month, day) = civil_from_days(days as i64);
    let hour = day_millis / 3_600_000;
    let minute = day_millis / 60_000 % 60;
    let second = day_millis / 1_000 % 60;
    let millisecond = day_millis % 1_000;
    IndexMap::from([
        ("year".into(), MiraValue::number(year as f64)),
        ("month".into(), MiraValue::number(month as f64)),
        ("day".into(), MiraValue::number(day as f64)),
        ("hour".into(), MiraValue::number(hour as f64)),
        ("minute".into(), MiraValue::number(minute as f64)),
        ("second".into(), MiraValue::number(second as f64)),
        ("millisecond".into(), MiraValue::number(millisecond as f64)),
        (
            "dayOfWeek".into(),
            MiraValue::number((days as i64 + 4).rem_euclid(7) as f64),
        ),
        ("offset".into(), MiraValue::number(offset)),
    ])
}

fn iso8601(timestamp: i64) -> String {
    let days = timestamp.div_euclid(86_400_000);
    let day_millis = timestamp.rem_euclid(86_400_000);
    let (year, month, day) = civil_from_days(days);
    let hour = day_millis / 3_600_000;
    let minute = day_millis / 60_000 % 60;
    let second = day_millis / 1_000 % 60;
    let millisecond = day_millis % 1_000;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millisecond:03}Z")
}

fn parse_iso8601(value: &str) -> Option<i64> {
    let (date, rest) = value.split_once('T').or_else(|| value.split_once(' '))?;
    let mut date = date.split('-');
    let year = date.next()?.parse::<i64>().ok()?;
    let month = date.next()?.parse::<u32>().ok()?;
    let day = date.next()?.parse::<u32>().ok()?;
    if date.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let (time, offset_minutes) = if let Some(time) = rest.strip_suffix('Z') {
        (time, 0i64)
    } else {
        let position = rest
            .char_indices()
            .skip(1)
            .find(|(_, character)| matches!(character, '+' | '-'))?
            .0;
        let (time, offset) = rest.split_at(position);
        let sign = if offset.starts_with('-') { -1 } else { 1 };
        let mut parts = offset[1..].split(':');
        let hours = parts.next()?.parse::<i64>().ok()?;
        let minutes = parts.next().unwrap_or("0").parse::<i64>().ok()?;
        if parts.next().is_some() || hours > 24 || minutes > 59 {
            return None;
        }
        (time, sign * (hours * 60 + minutes))
    };
    let mut parts = time.split(':');
    let hour = parts.next()?.parse::<i64>().ok()?;
    let minute = parts.next()?.parse::<i64>().ok()?;
    let second_part = parts.next()?;
    if parts.next().is_some() || hour > 23 || minute > 59 {
        return None;
    }
    let (second, milliseconds) = if let Some((second, fraction)) = second_part.split_once('.') {
        let second = second.parse::<i64>().ok()?;
        let fraction = fraction.chars().take(3).collect::<String>();
        let milliseconds = fraction.parse::<i64>().ok()? * 10i64.pow((3 - fraction.len()) as u32);
        (second, milliseconds)
    } else {
        (second_part.parse::<i64>().ok()?, 0)
    };
    if second > 59 {
        return None;
    }
    let days = days_from_civil(year, month, day);
    Some(
        days.checked_mul(86_400_000)?
            + hour * 3_600_000
            + minute * 60_000
            + second * 1_000
            + milliseconds
            - offset_minutes * 60_000,
    )
}

fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let month_prime = month as i64 + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + day as i64 - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let days = days + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year, month as u32, day as u32)
}
