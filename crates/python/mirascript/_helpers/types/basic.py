from __future__ import annotations
from typing_extensions import TYPE_CHECKING, overload

from ..constants import Uninitialized, kVmScript, kVmContext, kVmFunction
from ..._vm.types.module import VmModule
from ..._vm.types.wrapper import VmWrapper

if TYPE_CHECKING:

    from typing_extensions import Literal, TypeIs
    from ..._compiler import VmScript
    from ..._vm.types import (
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


def is_vm_wrapper(value) -> TypeIs[VmWrapper]:
    """检查值是否为 Mirascript 包装器"""
    return isinstance(value, VmWrapper)


def is_vm_module(value) -> TypeIs[VmModule]:
    """检查值是否为 Mirascript 模块"""
    return isinstance(value, VmModule)


def is_vm_extern(value) -> TypeIs[VmExtern]:
    """
    检查值是否为 Mirascript 外部对象

    Python 环境暂不支持外部对象，该函数目前总是返回 `False`
    """
    return False


def is_vm_callable(value) -> TypeIs[VmFunction | VmExtern]:
    """检查值是否为 Mirascript 可调用对象"""
    # Python 环境暂不支持外部对象，该函数目前等价于 `is_vm_function`
    return is_vm_function(value)


def is_vm_primitive(value) -> TypeIs[VmPrimitive]:
    """检查值是否为 Mirascript 原始值"""
    return value is None or type(value) in (str, int, float, bool)


def is_vm_array(value: VmAny) -> TypeIs[VmArray]:
    """检查值是否为 Mirascript 数组"""
    return type(value) is list


def is_vm_record(value: VmAny) -> TypeIs[VmRecord]:
    """检查值是否为 Mirascript 记录"""
    return type(value) is dict
