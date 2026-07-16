from ..._helpers.convert import to_boolean, to_format, to_number, to_string
from ..types.types import VmAny
from .common import AssertInit


def ToBoolean(value: VmAny) -> bool:
    AssertInit(value)
    return to_boolean(value)


def ToNumber(value: VmAny) -> float:
    AssertInit(value)
    return to_number(value)


def ToString(val: VmAny) -> str:
    AssertInit(val)
    if val is None:
        return ""
    return to_string(val)


def Format(val: VmAny, fmt: "str | None" = None) -> str:
    AssertInit(val)

    return to_format(val, fmt)
