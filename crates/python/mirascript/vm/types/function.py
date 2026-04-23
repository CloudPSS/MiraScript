# VmFunction 相关实现，Python 版
from functools import wraps
from ..helpers import CpEnter, CpExit

from .const import kVmFunction


def VmFunction(fn, option=None):
    if not callable(fn):
        raise TypeError("Invalid function")
    option = option or {}
    if option.get("injectCp"):
        original = fn

        @wraps(original)
        def wrapped(*args):
            try:
                CpEnter()
                return original(*args)
            finally:
                CpExit()

        fn = wrapped

        setattr(
            fn,
            kVmFunction,
            option.get("fullName", getattr(original, "__name__", "unknown")),
        )

    return fn
