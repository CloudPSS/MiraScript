import math
from typing_extensions import TypeVar, overload

from ..._vm.types.types import Uninitialized, VmAny, VmValue
from ..._vm.error import VmError
from ..serialize import display_function


def _number_to_string(x: "float | int") -> str:
    # 1. If x is nan, return "nan"
    if math.isnan(x):
        return "nan"

    # 2. If x is either +0 or -0, return "0"
    if x == 0:
        return "0"

    if x < 0:
        return "-" + _number_to_string(-x)

    if math.isinf(x):
        return "inf"

    # 5. Python 的 repr() 已经给出了符合规范的最短表示
    # 直接返回（Python 的 repr 遵循与 ECMAScript 相同的 IEEE 754 规则）
    result = repr(x)

    # 去除整数的 .0 后缀（Python 对整数值浮点数会添加 .0）
    if result.endswith(".0") and "e" not in result.lower():
        result = result[:-2]

    return result


def _inner_to_string(val: VmValue, useBraces: bool) -> str:
    if val is None:
        return "nil"
    if isinstance(val, bool):
        return "true" if val else "false"
    if isinstance(val, (int, float)):
        return _number_to_string(val)
    if callable(val):
        return display_function(val)

    if isinstance(val, (list, tuple)):
        strings = []
        for v in val:
            strings.append(_inner_to_string(v, True))
        joined = (", ").join(strings)
        if not useBraces:
            return joined
        return f"[{joined}]"

    if isinstance(val, dict):
        strings = []

        for k, v in val.items():
            strings.append(f"{k}: {_inner_to_string(v,True)}")
        joined = (", ").join(strings)
        if not useBraces:
            return joined
        return f"({joined})"
    return str(val)


T = TypeVar("T")


@overload
def to_string(value: VmAny) -> str: ...
@overload
def to_string(value: VmAny, fallback: T) -> "str | T": ...
def to_string(value: VmAny, fallback: T = Uninitialized) -> "str | T":
    if value is None or value is Uninitialized:
        return ""
    if isinstance(value, str):
        return value
    try:
        x = _inner_to_string(value, False)
        return x
    except Exception as ex:
        if fallback is Uninitialized:
            e = VmError(f"Cannot convert to string: {value}", "")
            e.__cause__ = ex
            raise e
        return fallback
