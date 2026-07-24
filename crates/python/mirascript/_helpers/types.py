from __future__ import annotations
from typing_extensions import TYPE_CHECKING, overload

from .constants import Uninitialized, kVmScript, kVmContext, kVmFunction
from .._vm.types.module import VmModule
from .._vm.types.wrapper import VmWrapper

if TYPE_CHECKING:

    from typing_extensions import Literal, TypeIs
    from .._compiler import VmScript
    from .._vm.types import (
        VmFunction,
        VmContext,
        VmExtern,
        VmPrimitive,
        VmRecord,
        VmArray,
        VmConst,
        VmImmutable,
        VmValue,
        VmAny,
    )


def is_vm_script(value) -> TypeIs[VmScript]:
    """检查是否为 Mirascript 脚本"""
    return callable(value) and getattr(value, kVmScript, False)


def is_vm_context(context) -> TypeIs[VmContext]:
    """检查是否为执行上下文"""
    return context is not None and getattr(context, kVmContext, False)


def is_vm_function(value) -> TypeIs[VmFunction]:
    """检查是否为 Mirascript 函数"""
    return callable(value) and getattr(value, kVmFunction, None) is not None


def is_vm_module(value) -> TypeIs[VmModule]:
    """检查值是否为 Mirascript 模块"""
    return isinstance(value, VmModule)


def is_vm_extern(value) -> TypeIs[VmExtern]:
    """
    检查值是否为 Mirascript 外部对象

    Python 环境暂不支持外部对象，该函数目前总是返回 `False`
    """
    return False


def is_vm_wrapper(value) -> TypeIs[VmWrapper]:
    """检查值是否为 Mirascript 包装器"""
    return isinstance(value, VmWrapper)


def is_vm_callable(value) -> TypeIs[VmFunction | VmExtern]:
    """检查值是否为 Mirascript 可调用对象"""
    # Python 环境暂不支持外部对象，该函数目前等价于 `is_vm_function`
    return is_vm_function(value)


def is_vm_primitive(value) -> TypeIs[VmPrimitive]:
    """检查值是否为 Mirascript 原始值"""
    return isinstance(value, (str, int, float, bool)) or value is None


def is_vm_array(value: VmAny) -> TypeIs[VmArray]:
    """检查值是否为 Mirascript 数组"""
    return isinstance(value, list)


def is_vm_record(value: VmAny) -> TypeIs[VmRecord]:
    """检查值是否为 Mirascript 记录"""
    return isinstance(value, dict)


def _is_vm_const(value, depth: int) -> TypeIs[VmConst]:
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
        inner_depth = depth - 1
        if is_vm_array(value):
            return all(_is_vm_const(v, inner_depth) for v in value)
        if is_vm_record(value):
            return all(
                _is_vm_const(v, inner_depth) and isinstance(k, str)
                for k, v in value.items()
            )
        return False


@overload
def is_vm_const(
    value: VmAny, check_deep: Literal[False] = False
) -> TypeIs[VmConst]: ...
@overload
def is_vm_const(value, check_deep: Literal[True]) -> TypeIs[VmConst]: ...
def is_vm_const(value, check_deep=False):
    """检查值是否为 Mirascript 常量"""
    return _is_vm_const(value, 16 if check_deep else 0)


@overload
def is_vm_immutable(
    value: VmAny, check_deep: Literal[False] = False
) -> TypeIs[VmImmutable]: ...
@overload
def is_vm_immutable(value, check_deep: Literal[True]) -> TypeIs[VmImmutable]: ...
def is_vm_immutable(value, check_deep=False) -> TypeIs[VmImmutable]:
    """检查值是否为 Mirascript 不可变值"""
    return (
        is_vm_const(value, check_deep) or is_vm_function(value) or is_vm_module(value)
    )


def is_vm_any(value, check_deep: bool) -> TypeIs[VmAny]:
    """检查值是否为 Mirascript 值（包括未初始化变量）"""
    if value is Uninitialized:
        return True
    if is_vm_callable(value) or is_vm_wrapper(value):
        return True
    return is_vm_const(value, check_deep)


@overload
def is_vm_value(
    value: VmAny, check_deep: Literal[False] = False
) -> TypeIs[VmValue]: ...
@overload
def is_vm_value(value, check_deep: Literal[True]) -> TypeIs[VmValue]: ...
def is_vm_value(value, check_deep=False) -> TypeIs[VmValue]:
    """检查值是否为 Mirascript 合法值"""
    if value is Uninitialized:
        return False
    return is_vm_any(value, check_deep)


def get_vm_type(
    value: VmAny,
) -> Literal[
    "uninitialized",
    "boolean",
    "number",
    "string",
    "record",
    "array",
    "function",
    "module",
    "extern",
    "unknown",
]:
    """获取 Mirascript 类型"""
    if value is Uninitialized:
        return "uninitialized"
    if isinstance(value, bool):
        return "boolean"
    if isinstance(value, (int, float)):
        return "number"
    if isinstance(value, str):
        return "string"
    if is_vm_record(value):
        return "record"
    if is_vm_array(value):
        return "array"
    if is_vm_function(value):
        return "function"
    if is_vm_module(value):
        return "module"
    if is_vm_extern(value):
        return "extern"
    return "unknown"
