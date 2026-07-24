from __future__ import annotations
from typing_extensions import TypeVar, overload
import math
import sys

from ..._vm.types import Uninitialized, VmAny, VmValue
from ..._vm.error import VmError
from ..serialize import display

MAX_INTEGER = 1e21


if sys.version_info >= (3, 12):

    def _is_integer(x: float | int) -> bool:
        return x.is_integer()

else:

    def _is_integer(x: float | int) -> bool:
        return isinstance(x, int) or (isinstance(x, float) and x.is_integer())


def number_to_string(x: float | int) -> str:
    # 1. Fast path for integers, including +-0
    if _is_integer(x) and -MAX_INTEGER < x < MAX_INTEGER:
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


def _inner_to_string(val: VmValue, useBraces: bool) -> str:
    if val is None:
        return "nil"
    if isinstance(val, bool):
        return "true" if val else "false"
    if isinstance(val, (int, float)):
        return number_to_string(val)
    if callable(val):
        return display(val)

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
def to_string(value: VmAny, fallback: T) -> str | T: ...
def to_string(value: VmAny, fallback: T = Uninitialized) -> str | T:
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
