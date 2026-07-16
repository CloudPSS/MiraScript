from ..._helpers.convert.to_boolean import toBoolean
from ..._helpers.convert.to_format import toFormat
from ..._helpers.convert.to_number import toNumber
from ..._helpers.convert.to_string import toString
from ..types.types import VmAny
from .common import AssertInit


def ToBoolean(value: VmAny) -> bool:
    AssertInit(value)
    return toBoolean(value)


def ToNumber(value: VmAny) -> float:
    AssertInit(value)
    return toNumber(value)


def ToString(val: VmAny) -> str:
    AssertInit(val)
    if val is None:
        return ""
    return toString(val)


def Format(val: VmAny, fmt: "str | None" = None) -> str:
    AssertInit(val)

    return toFormat(val, fmt)
