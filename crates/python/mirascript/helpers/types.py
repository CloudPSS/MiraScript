from typing_extensions import (
    TypeGuard,
    TYPE_CHECKING,
    overload,
    Literal,
)

from .constants import kVmScript, kVmContext, kVmFunction
from ..vm.types.module import VmModule
from ..vm.types.types import Uninitialized

if TYPE_CHECKING:
    from ..compiler import VmScript
    from ..vm.types.types import *
    from ..vm.types.context import VmContext


def is_vm_script(value) -> "TypeGuard[VmScript]":
    """检查是否为 Mirascript 脚本"""
    return callable(value) and getattr(value, kVmScript, False)


def is_vm_context(context) -> "TypeGuard[VmContext]":
    """检查是否为执行上下文"""
    return context is not None and getattr(context, kVmContext, False)


def is_vm_function(value) -> "TypeGuard[VmFunction]":
    """检查是否为 Mirascript 函数"""
    return callable(value) and getattr(value, kVmFunction, False)


def is_vm_module(value) -> "TypeGuard[VmModule]":
    """检查值是否为 Mirascript 模块"""
    return isinstance(value, VmModule)


def is_vm_extern(value) -> "TypeGuard[VmExtern]":
    """
    检查值是否为 Mirascript 外部对象

    Python 环境暂不支持外部对象，该函数目前总是返回 `False`
    """
    return False


def is_vm_wrapper(value) -> "TypeGuard[VmModule | VmExtern]":
    """检查值是否为 Mirascript 包装器"""
    return is_vm_module(value) or is_vm_extern(value)


def is_vm_callable(value) -> "TypeGuard[VmFunction | VmExtern]":
    """检查值是否为 Mirascript 可调用对象"""
    if is_vm_extern(value):
        return False  # 外部对象暂不支持调用
    return is_vm_function(value)


def is_vm_primitive(value) -> "TypeGuard[VmPrimitive]":
    """检查值是否为 Mirascript 原始值"""
    return isinstance(value, (str, int, float, bool)) or value is None


def is_vm_array(value: "VmAny") -> "TypeGuard[VmArray]":
    """检查值是否为 Mirascript 数组"""
    return isinstance(value, list)


def is_vm_record(value: "VmAny") -> "TypeGuard[VmRecord]":
    """检查值是否为 Mirascript 记录"""
    return isinstance(value, dict)


def _is_vm_const(value, depth: int) -> "TypeGuard[VmConst]":
    """检查值是否为 Mirascript 常量"""
    if is_vm_function(value) or is_vm_wrapper(value):
        return False
    if is_vm_primitive(value):
        return True
    if depth <= 0:
        if is_vm_array(value) or is_vm_record(value):
            return True
        return False
    else:
        if is_vm_array(value):
            return all(_is_vm_const(v, depth - 1) for v in value)
        if is_vm_record(value):
            return all(_is_vm_const(v, depth - 1) for v in value.values())
        return False


@overload
def is_vm_const(
    value: "VmAny", check_deep: Literal[False] = False
) -> "TypeGuard[VmConst]": ...
@overload
def is_vm_const(value, check_deep: Literal[True]) -> "TypeGuard[VmConst]": ...
def is_vm_const(value, check_deep=False):
    """检查值是否为 Mirascript 常量"""
    return _is_vm_const(value, 16 if check_deep else 0)


def is_vm_immutable(value) -> "TypeGuard[VmImmutable]":
    """检查值是否为 Mirascript 不可变值"""
    return is_vm_const(value) or is_vm_function(value) or is_vm_module(value)


def is_vm_any(value, check_deep: bool) -> "TypeGuard[VmAny]":
    """检查值是否为 Mirascript 值（包括未初始化变量）"""
    if value is Uninitialized:
        return True
    if is_vm_callable(value) or is_vm_wrapper(value):
        return True
    return is_vm_const(value, check_deep)


@overload
def is_vm_value(
    value: "VmAny", check_deep: Literal[False] = False
) -> "TypeGuard[VmValue]": ...
@overload
def is_vm_value(value, check_deep: Literal[True]) -> "TypeGuard[VmValue]": ...
def is_vm_value(value, check_deep=False):
    """检查值是否为 Mirascript 合法值"""
    if value is Uninitialized:
        return False
    return is_vm_any(value, check_deep)
