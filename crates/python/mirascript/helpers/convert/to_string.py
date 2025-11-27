import math
from mirascript.vm.error import VmError
from mirascript.vm.types.const import Uninitialized, getVmFunctionInfo
from mirascript.vm.types.wrapper import VmWrapper, isVmWrapper


def numberToString_(x) -> str:

    # 1. If x is nan, return "nan"
    if math.isnan(x):
        return "nan"

    # 2. If x is either +0 or -0, return "0"
    if x == 0:
        return "0"

    if x < 0:
        return "-" + numberToString_(-x)

    if math.isinf(x):
        return "inf"

    # 5. Python 的 repr() 已经给出了符合规范的最短表示
    # 直接返回（Python 的 repr 遵循与 ECMAScript 相同的 IEEE 754 规则）
    result = repr(x)

    # 去除整数的 .0 后缀（Python 对整数值浮点数会添加 .0）
    if result.endswith(".0") and "e" not in result.lower():
        result = result[:-2]

    return result


def innerToString_(val, useBraces) -> str:
    if val is None:
        return "nil"

    if callable(val):
        name = getVmFunctionInfo(val)
        if name:
            return f"<function {name}>"
        return "<function>"

    if isVmWrapper(val):
        return val.toString()

    if isinstance(val, (list, tuple)):
        strings = []
        for v in val:
            strings.append(innerToString_(v, True))
        joined = (", ").join(strings)
        if not useBraces:
            return joined
        return f"[{joined}]"

    if isinstance(val, dict):
        strings = []

        for k, v in val.items():
            strings.append(f"{k}: {innerToString_(v,True)}")
        joined = (", ").join(strings)
        if not useBraces:
            return joined
        return f"({joined})"
    if isinstance(val, bool):
        return "true" if val else "false"
        # return "true" if val else "0"

    if isinstance(val, (int, float)):

        return numberToString_(val)
    return str(val)


def toString(value, fallback=Uninitialized, useBraces=False):
    if value is None:
        return ""
    if isinstance(value, str):
        return value
    try:
        x= innerToString_(value, useBraces)
        return x
    except Exception as ex:
        if fallback is Uninitialized:
            e= VmError(f"Cannot convert to string: {value}", "") 
            e.__cause__ = ex
            raise e
        return fallback
