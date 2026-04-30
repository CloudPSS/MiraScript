import math


def is_safe_integer(num: "float | int") -> bool:
    """
    检查是否为安全整数（在 64 位浮点数精确表示范围内）
    类似于 JavaScript 的 Number.isSafeInteger()
    """

    num = float(num)
    if not num.is_integer():
        return False

    MIN_SAFE_INTEGER = -(2**53) + 1
    MAX_SAFE_INTEGER = 2**53 - 1
    return MIN_SAFE_INTEGER <= num <= MAX_SAFE_INTEGER
