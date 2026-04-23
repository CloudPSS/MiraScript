from dataclasses import dataclass
from typing import Any, Callable, Dict, Optional, Union
from .module import VmModule
from .extern import VmExtern

## 标记未初始化的值
Uninitialized = type("Uninitialized", (), {})()


## 标记当前未返回的值
NotReturned = type("NotReturned", (), {})()


# 类型别名
VmPrimitive = Union[type(None), str, int, float, bool]
        
VmRecord = dict
VmArray = list
VmConst = Union[type(None), str, int, float, bool, dict, list]



VM_ARRAY_MAX_LENGTH = 2 ** 31 - 1

class VmFunctionLike():
    def __call__(self, *args): ... # type: ignore


VmImmutable = Union[ VmConst ,  VmModule]

VmValue =Union[ VmImmutable , VmExtern]
VmUninitialized = type(Uninitialized)
VmAny = Union[VmValue , VmUninitialized]

kVmFunction = 'mirascript_vm_function'
def getVmFunctionInfo(value) :
    if (not callable(value)):
        return None
    return getattr(value, kVmFunction, None)