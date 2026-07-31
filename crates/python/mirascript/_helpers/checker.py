from __future__ import annotations
from typing_extensions import TypeGuard, Any


def is_number(a: Any) -> TypeGuard[int | float]:
    return type(a) in (int, float)


_MIN_SAFE_INTEGER = -(2**53) + 1
_MAX_SAFE_INTEGER = 2**53 - 1


def is_safe_integer(num: float | int) -> bool:
    """
    检查是否为安全整数（在 64 位浮点数精确表示范围内）
    类似于 JavaScript 的 Number.isSafeInteger()
    """
    if isinstance(num, int):
        return _MIN_SAFE_INTEGER <= num <= _MAX_SAFE_INTEGER

    return num.is_integer() and _MIN_SAFE_INTEGER <= num <= _MAX_SAFE_INTEGER
