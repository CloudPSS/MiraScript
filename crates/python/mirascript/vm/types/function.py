
# VmFunction 相关实现，Python 版
from dataclasses import dataclass
from functools import wraps
from typing import Callable, Any, Optional, Dict, Protocol, Tuple, TypeVar, TypedDict, Union, cast


from ..helpers import CpEnter, CpExit
from .const import VmAny, VmValue,VmFunctionLike,VmFunctionWrapper,VmFunctionInfo,VmFunctionOption
from .extern import VmExtern, wrap_to_vm_value, unwrap_from_vm_value 


def VmFunction(fn, option: Union[VmFunctionOption, None] = None) -> VmFunctionLike:
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

    return fn

