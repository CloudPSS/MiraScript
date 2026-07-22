from __future__ import annotations
import math
from time import time
from datetime import datetime as datetime_dt, timezone

from ...._helpers.checker import is_number
from ...._helpers.convert import to_number
from ...._helpers.serialize import display
from ...types import Uninitialized
from .._helpers import (
    _describe_param,
    _expect_number_range,
    _throw_error,
    _throw_unexpected_type_error,
)

__all__ = ["to_timestamp", "to_datetime", "to_iso8601"]


def _from_number(datetime: float | int, fallback: bool) -> float | None:
    try:
        # 尝试将数值转换为datetime对象并获取时间戳
        dt = datetime_dt.fromtimestamp(datetime / 1000.0, tz=timezone.utc)
        return dt.timestamp() * 1000.0
    except (ValueError, OSError, OverflowError):
        if fallback:
            return None
        return _throw_error(
            f"{_describe_param('datetime')} is an invalid timestamp: {display(datetime)}",
            math.nan,
        )


def _timestamp(datetime, fallback: bool) -> float | None:
    if datetime is None:
        return time() * 1000.0
    if is_number(datetime):
        return _from_number(datetime, fallback)
    if not isinstance(datetime, str):
        if fallback:
            return None
        _throw_unexpected_type_error("datetime", "number | string", datetime, math.nan)

    num = to_number(datetime, math.nan)
    if not math.isnan(num):
        return _from_number(num, fallback)

    try:
        d = datetime_dt.fromisoformat(datetime)
        return d.timestamp() * 1000
    except Exception:
        # 尝试解析常见 RFC 2822/822 日期（如 'Wed, 02 Oct 2002 08:00:00 GMT'）
        try:
            from email.utils import parsedate_to_datetime

            d = parsedate_to_datetime(datetime)
            return d.timestamp() * 1000
        except Exception:
            if fallback:
                return None
            return _throw_error(
                f"{_describe_param('datetime')} cannot be parsed as datetime:  {display(datetime)}",
                math.nan,
            )


def to_timestamp(dt=Uninitialized, fallback=Uninitialized):
    """将输入转换为 Unix 毫秒时间戳。"""
    # 未传入或显式未初始化 -> 当前时间（毫秒）
    timestamp = _timestamp(dt, fallback is not Uninitialized)
    if timestamp is None:
        return fallback
    return timestamp


def to_datetime(
    datetime_value=Uninitialized, offset=Uninitialized, fallback=Uninitialized
):
    """将输入转换为 Date 记录（UTC），并应用小时为单位的偏移量。"""
    timestamp = _timestamp(datetime_value, fallback is not Uninitialized)
    if timestamp is None:
        return fallback
    # 解析 offset（小时），若不可用或不是有限数则视为 0
    o = _expect_number_range(
        "offset", 0 if offset is Uninitialized or offset is None else offset, -24, 24
    )

    # 构造 UTC 时间并应用偏移（小时）
    utc_seconds = timestamp / 1000.0 + float(o) * 3600.0
    d = datetime_dt.fromtimestamp(utc_seconds, tz=timezone.utc)

    # JS getUTCDay(): 0 (Sunday) .. 6 (Saturday)
    # Python weekday(): 0 (Monday) .. 6 (Sunday)
    day_of_week_js = (d.weekday() + 1) % 7

    return {
        "year": d.year,
        "month": d.month,
        "day": d.day,
        "hour": d.hour,
        "minute": d.minute,
        "second": d.second,
        "millisecond": int(d.microsecond / 1000),
        "dayOfWeek": day_of_week_js,
        "offset": float(o),
    }


def to_iso8601(datetime=Uninitialized, fallback=Uninitialized):
    """将输入转换为 ISO 8601 字符串（UTC，精确到毫秒）。"""
    timestamp = _timestamp(datetime, fallback is not Uninitialized)
    if timestamp is None:
        return fallback

    d = datetime_dt.fromtimestamp(timestamp / 1000.0, tz=timezone.utc)
    # 格式化到毫秒，与 JS Date.prototype.toISOString() 风格一致
    iso = d.strftime("%Y-%m-%dT%H:%M:%S.%f")[:-3] + "Z"
    return iso
