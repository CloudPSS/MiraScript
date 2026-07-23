from __future__ import annotations

from ..._helpers.convert import to_boolean, to_format, to_number, to_string
from ..types import VmAny
from .common import AssertInit


def ToBoolean(value: VmAny) -> bool:
    if isinstance(value, bool):
        return value
    AssertInit(value)
    return to_boolean(value)


def ToNumber(value: VmAny) -> float:
    if isinstance(value, float):
        return value
    AssertInit(value)
    return to_number(value)


def ToString(val: VmAny) -> str:
    if isinstance(val, str):
        return val
    AssertInit(val)
    if val is None:
        return ""
    return to_string(val)


def Format(val: VmAny, fmt: str | None = None) -> str:
    if isinstance(val, str):
        return val
    AssertInit(val)

    return to_format(val, fmt)
