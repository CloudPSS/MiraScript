//! Time-related global functions.

use indexmap::IndexMap;
use time::{Duration, Timestamp};

use crate::{MiraError, MiraType, MiraValue, Result, Runtime, RuntimeErrorKind, operations};

use crate::standard_library::global_builtin;

pub(super) fn install(runtime: &mut Runtime) {
    global_builtin!(runtime, fn to_timestamp(call, args) {
        match timestamp(call, args.first()) {
            Ok(value) => Ok(MiraValue::number(value.as_milliseconds() as f64)),
            Err(_) if args.len() > 1 => Ok(args[1]),
            Err(error) => Err(error),
        }
    });
    global_builtin!(runtime, fn to_datetime(call, args) {
        let timestamp = match timestamp(call, args.first()) {
            Ok(value) => value,
            Err(_) if args.len() > 2 => return Ok(args[2]),
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

    global_builtin!(runtime, fn to_iso8601(call, args) {
        match timestamp(call, args.first()) {
            Ok(value) => {
                const FMT_CONFIG: time::format_description::well_known::iso8601::EncodedConfig =
                    time::format_description::well_known::iso8601::Config::DEFAULT
                        .set_time_precision(
                            time::format_description::well_known::iso8601::TimePrecision::Second {
                                decimal_digits: Some(std::num::NonZeroU8::new(3u8).unwrap()),
                            },
                        )
                        .encode();
                let format = time::format_description::well_known::Iso8601::<FMT_CONFIG>;

                call.insert(value.format(&format).map_err(|_| {
                    MiraError::runtime(RuntimeErrorKind::InvalidTimestamp {
                        actual: MiraType::Number,
                    })
                })?)
            }
            Err(_) if args.len() > 1 => Ok(args[1]),
            Err(error) => Err(error),
        }
    });
}

fn now(call: &mut crate::Runtime) -> Result<Timestamp> {
    let i = call.options().providers.now_millis();
    Timestamp::from_milliseconds(i).map_err(|_| {
        MiraError::runtime(RuntimeErrorKind::InvalidTimestamp {
            actual: MiraType::Nil,
        })
    })
}

fn timestamp(call: &mut crate::Runtime, value: Option<&MiraValue>) -> Result<Timestamp> {
    let Some(value) = value else {
        return now(call);
    };
    if value.is_nil() {
        return now(call);
    }
    if let Some(number) = value.as_number()
        && number < i64::MAX as f64
        && number > i64::MIN as f64
    {
        return Timestamp::from_milliseconds(number.trunc() as i64).map_err(|_| {
            MiraError::runtime(RuntimeErrorKind::InvalidTimestamp {
                actual: MiraType::Number,
            })
        });
    }
    if let Some(str) = value.as_str(call)? {
        if let Ok(number) = operations::to_number(call, *value)
            && number < i64::MAX as f64
            && number > i64::MIN as f64
        {
            return Timestamp::from_milliseconds(number.trunc() as i64).map_err(|_| {
                MiraError::runtime(RuntimeErrorKind::InvalidTimestamp {
                    actual: MiraType::String,
                })
            });
        }
        return Timestamp::parse(str, &time::format_description::well_known::Iso8601::PARSING)
            .or_else(|_| Timestamp::parse(str, &time::format_description::well_known::Rfc2822))
            .or_else(|_| Timestamp::parse(str, &time::format_description::well_known::Rfc3339))
            .map_err(|_| {
                MiraError::runtime(RuntimeErrorKind::InvalidTimestamp {
                    actual: MiraType::String,
                })
            });
    }
    Err(MiraError::runtime(RuntimeErrorKind::InvalidTimestamp {
        actual: value.value_type(),
    }))
}

fn datetime_record(timestamp: Timestamp, offset: f64) -> IndexMap<String, MiraValue> {
    // let adjusted = timestamp as i128 + (offset * 3_600_000.0).trunc() as i128;
    let adjusted = timestamp + Duration::seconds_f64(offset * 3_600.0);
    let adjusted = adjusted.to_utc();

    IndexMap::from([
        ("year".into(), MiraValue::number(adjusted.year() as f64)),
        (
            "month".into(),
            MiraValue::number(adjusted.month() as u8 as f64),
        ),
        ("day".into(), MiraValue::number(adjusted.day() as f64)),
        ("hour".into(), MiraValue::number(adjusted.hour() as f64)),
        ("minute".into(), MiraValue::number(adjusted.minute() as f64)),
        ("second".into(), MiraValue::number(adjusted.second() as f64)),
        (
            "millisecond".into(),
            MiraValue::number(adjusted.millisecond() as f64),
        ),
        (
            "dayOfWeek".into(),
            MiraValue::number((((adjusted.weekday() as u8) + 1) % 7) as f64),
        ),
        ("offset".into(), MiraValue::number(offset)),
    ])
}
