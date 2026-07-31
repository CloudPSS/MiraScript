from __future__ import annotations
from typing_extensions import TypeVar, overload, TYPE_CHECKING
import math

from ..constants import Uninitialized

if TYPE_CHECKING:
    from ..._vm.types import VmAny, VmValue

    T = TypeVar("T")

_MAX_INTEGER = 1e21


def is_json_integer(num: float | int) -> bool:
    """
    检查是否为 JSON 整数
    """
    if isinstance(num, int):
        return True
    return num.is_integer() and -_MAX_INTEGER < num < _MAX_INTEGER


def number_to_string(x: float | int, minus_zero: bool = False) -> str:
    # 0. If x is -0, return "-0"
    if minus_zero and x == 0 and math.copysign(1, x) < 0:
        return "-0"

    # 1. Fast path for integers, including +-0
    if is_json_integer(x):
        return repr(int(x))

    x = float(x)  # Convert to float

    # 2. If x is nan, return "nan"
    if math.isnan(x):
        return "nan"

    # 3. If x is +-inf, return "inf" or "-inf"
    if math.isinf(x):
        return "inf" if x > 0 else "-inf"

    # 4. return the string representation of the float
    result = repr(x)
    # Note: no need to remove trailing ".0" since integers are handled in step 1

    # 5. fix 1e-09 -> 1e-9
    result = result.replace("e-0", "e-")

    return result


def inner_to_string(val: VmValue, useBraces: bool) -> str:
    if val is None:
        return "nil"
    if isinstance(val, bool):
        return "true" if val else "false"
    if isinstance(val, (int, float)):
        return number_to_string(val)
    if callable(val):
        from ..serialize import display

        return display(val)

    if isinstance(val, (list, tuple)):
        strings = []
        for v in val:
            strings.append(inner_to_string(v, True))
        joined = (", ").join(strings)
        if not useBraces:
            return joined
        return f"[{joined}]"

    if isinstance(val, dict):
        strings = []

        for k, v in val.items():
            strings.append(f"{k}: {inner_to_string(v,True)}")
        joined = (", ").join(strings)
        if not useBraces:
            return joined
        return f"({joined})"
    return str(val)


@overload
def to_string(value: VmAny) -> str: ...
@overload
def to_string(value: VmAny, fallback: T) -> str | T: ...
def to_string(value: VmAny, fallback: T = Uninitialized) -> str | T:
    if value is None or value is Uninitialized:
        return ""
    if isinstance(value, str):
        return value
    try:
        x = inner_to_string(value, False)
        return x
    except Exception as ex:
        if fallback is Uninitialized:
            from ..._vm.error import VmError

            e = VmError(f"Cannot convert to string: {value!r}", "")
            e.__cause__ = ex
            raise e
        return fallback
