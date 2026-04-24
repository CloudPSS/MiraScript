from typing import Union
import time
import math
from datetime import datetime as datetime_dt, timezone

from mirascript.helpers.convert.to_number import toNumber
from mirascript.helpers.serialize import display
from mirascript.vm.lib._helpers import (
    describeParam,
    _expect_number_range,
    _throw_error,
    throw_unexpected_type_error,
)
from mirascript.vm.types.const import Uninitialized


def fromNumber(datetime, fallback):
    """将数值转换为 Python 数值类型（int 或 float）。

    若数值为有限整数且在安全范围内，则返回 int 类型，否则返回 float 类型。
    """

    if math.isfinite(datetime):
        return datetime

    try:
        # 尝试将数值转换为datetime对象并获取时间戳
        dt = datetime_dt.fromtimestamp(datetime)
        return dt.timestamp()
    except (ValueError, OSError, OverflowError):
        # 转换失败时的处理
        if fallback:
            return None
        else:
            # 这里需要根据你的错误处理机制来调整
            # 假设你有一个throw_error函数
            _throw_error(
                f"{describeParam('datetime')} is an invalid timestamp: {display(datetime)}",
                math.nan,
            )


def getTimestamp(datetime, fallback) -> Union[float, None]:
    if datetime is None:
        return time.time() * 1000
    if isinstance(datetime, bool):
        if fallback:
            return None
        else:
            throw_unexpected_type_error(
                "datetime", "number | string", datetime, math.nan
            )

    if isinstance(datetime, (int, float)):
        return fromNumber(datetime, fallback)

    if not isinstance(datetime, str):
        if fallback:
            return None
        else:
            throw_unexpected_type_error(
                "datetime", "number | string", datetime, math.nan
            )

    num = toNumber(datetime, math.nan)
    if not math.isnan(num):
        return fromNumber(num, fallback)

    try:
        if datetime.endswith("Z"):
            s2 = datetime[:-1] + "+00:00"
        # Python 3.7+ 支持 fromisoformat 带时区偏移
        d = datetime_dt.fromisoformat(datetime)
        return d.timestamp() * 1000
    except Exception:
        # 尝试解析常见 RFC 2822/822 日期（如 'Wed, 02 Oct 2002 08:00:00 GMT'）
        try:
            from email.utils import parsedate_to_datetime

            d = parsedate_to_datetime(datetime)
            return d.timestamp() * 1000
        except Exception:
            # return float('nan')
            if fallback:
                return None
            _throw_error(
                f"{describeParam('datetime')} cannot be parsed as datetime:  {display(datetime)}",
                math.nan,
            )


def to_timestamp(dt=Uninitialized, fallback=Uninitialized):
    """将输入转换为 Unix 毫秒时间戳。

    行为与 TypeScript 版本保持一致：
    - dt 为 Uninitialized 或 None 时返回当前时间（毫秒）
    - 若为数字则视为毫秒时间戳直接返回
    - 否则先尝试将字符串转换为数字，再判断是否为有限数
    - 最后尝试解析 ISO / RFC 日期字符串，解析失败返回 NaN
    """
    # 未传入或显式未初始化 -> 当前时间（毫秒）
    timestamp = getTimestamp(dt, fallback is not Uninitialized)
    if timestamp is None:
        return fallback
    return timestamp


def to_datetime(
    datetime_value=Uninitialized, offset=Uninitialized, fallback=Uninitialized
):
    """将输入转换为 Date 记录（UTC），并应用小时为单位的偏移量。

    返回一个字典，字段与 TypeScript 版本保持一致。
    若无法解析时间则返回 None。
    """
    timestamp = getTimestamp(datetime_value, fallback is not Uninitialized)
    if timestamp is None:
        return fallback
    o = _expect_number_range(
        "offset", 0 if offset is Uninitialized or offset is None else offset, -24, 24
    )
    # 解析 offset（小时），若不可用或不是有限数则视为 0

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


def to_iso8601(datetime_value=Uninitialized, fallback=Uninitialized):
    """将输入转换为 ISO 8601 字符串（UTC，精确到毫秒）。

    解析失败返回 None。
    """
    timestamp = getTimestamp(datetime_value, fallback is not Uninitialized)
    if timestamp is None:
        return fallback

    d = datetime_dt.fromtimestamp(timestamp / 1000.0, tz=timezone.utc)
    # 格式化到毫秒，与 JS Date.prototype.toISOString() 风格一致
    iso = d.strftime("%Y-%m-%dT%H:%M:%S.%f")[:-3] + "Z"
    return iso
