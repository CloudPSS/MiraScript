
from mirascript.vm.types.const import Uninitialized
import time
import math
import datetime as _dt
from mirascript.vm.operations import ToString_, ToNumber_


def to_timestamp(dt=Uninitialized):
    """将输入转换为 Unix 毫秒时间戳。

    行为与 TypeScript 版本保持一致：
    - dt 为 Uninitialized 或 None 时返回当前时间（毫秒）
    - 若为数字则视为毫秒时间戳直接返回
    - 否则先尝试将字符串转换为数字，再判断是否为有限数
    - 最后尝试解析 ISO / RFC 日期字符串，解析失败返回 NaN
    """
    # 未传入或显式未初始化 -> 当前时间（毫秒）
    if dt is Uninitialized or dt is None:
        return time.time() * 1000

    # 数值直接视作毫秒时间戳
    if isinstance(dt, (int, float)):
        if not math.isfinite(dt):
            return math.nan
        return float(dt)

    # 先转换为字符串
    s = ToString_(dt)
    if not s:
        return float('nan')

    # 尝试将字符串转换为数字
    num = ToNumber_(s)
    if isinstance(num, (int, float)) and math.isfinite(num):
        return float(num)

    # 最后尝试解析为日期字符串（支持 ISO 8601 与常见 RFC 格式）
    # 将尾随的 Z 替换为 +00:00 以兼容 fromisoformat
    s2 = s
    try:
        if s2.endswith('Z'):
            s2 = s2[:-1] + '+00:00'
        # Python 3.7+ 支持 fromisoformat 带时区偏移
        d = _dt.datetime.fromisoformat(s2)
        return d.timestamp() * 1000
    except Exception:
        # 尝试解析常见 RFC 2822/822 日期（如 'Wed, 02 Oct 2002 08:00:00 GMT'）
        try:
            from email.utils import parsedate_to_datetime

            d = parsedate_to_datetime(s)
            return d.timestamp() * 1000
        except Exception:
            return float('nan')


def to_datetime(datetime_value=Uninitialized, offset=None):
    """将输入转换为 Date 记录（UTC），并应用小时为单位的偏移量。

    返回一个字典，字段与 TypeScript 版本保持一致。
    若无法解析时间则返回 None。
    """
    ts = to_timestamp(datetime_value)
    if not isinstance(ts, (int, float)) or not math.isfinite(ts):
        return None

    # 解析 offset（小时），若不可用或不是有限数则视为 0
    o = ToNumber_(offset if offset is not None else 0)
    if not isinstance(o, (int, float)) or not math.isfinite(o):
        o = 0
    # 构造 UTC 时间并应用偏移（小时）
    utc_seconds = ts / 1000.0 + float(o) * 3600.0
    d = _dt.datetime.utcfromtimestamp(utc_seconds)

    # JS getUTCDay(): 0 (Sunday) .. 6 (Saturday)
    # Python weekday(): 0 (Monday) .. 6 (Sunday)
    day_of_week_js = (d.weekday() + 1) % 7

    return {
        'year': d.year,
        'month': d.month,
        'day': d.day,
        'hour': d.hour,
        'minute': d.minute,
        'second': d.second,
        'millisecond': int(d.microsecond / 1000),
        'dayOfWeek': day_of_week_js,
        'offset': float(o),
    }


def to_iso8601(datetime_value=Uninitialized):
    """将输入转换为 ISO 8601 字符串（UTC，精确到毫秒）。

    解析失败返回 None。
    """
    ts = to_timestamp(datetime_value)
    if not isinstance(ts, (int, float)) or not math.isfinite(ts):
        return None

    d = _dt.datetime.utcfromtimestamp(ts / 1000.0)
    # 格式化到毫秒，与 JS Date.prototype.toISOString() 风格一致
    iso = d.strftime('%Y-%m-%dT%H:%M:%S.%f')[:-3] + 'Z'
    return iso


