from dataclasses import dataclass
from typing import Any, Callable, Dict, Optional, Protocol, TypedDict, Union
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

class VmFunctionLike(Protocol):
    def __call__(self, *args): ... # type: ignore


# Mirascript 函数信息
class VmFunctionInfo(TypedDict, total=False):
    fullName: str
    isLib: bool
    summary: Optional[str]
    params: Optional[Dict[str, str]]
    paramsType: Optional[Dict[str, str]]
    returns: Optional[str]
    returnsType: Optional[str]
    original: Optional[VmFunctionLike]

# Mirascript 函数创建选项
class VmFunctionOption(TypedDict, total=False):
    fullName: str
    isLib: bool
    summary: str
    params: Dict[str, str]
    paramsType: Dict[str, str]
    returns: str
    returnsType: str
    injectCp: bool
  
# 我们在 Python 中用一个 wrapper class 来模拟 TypeScript 的函数+属性的组合
@dataclass
class VmFunctionWrapper:
    func: VmFunctionLike
    info: VmFunctionInfo
    proxy: Optional[Callable[..., Any]] = None
    def __call__(self, *args):
        return self.func(*args)

VmImmutable = Union[ VmConst , VmFunctionWrapper , VmModule]

VmValue =Union[ VmImmutable , VmExtern]
VmUninitialized = type(Uninitialized)
VmAny = Union[VmValue , VmUninitialized]

kVmFunction = 'mirascript_vm_function'
def getVmFunctionInfo(value) :
    if (not callable(value)):
        return None
    return getattr(value, kVmFunction, None)