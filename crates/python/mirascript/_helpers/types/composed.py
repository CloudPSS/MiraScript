from __future__ import annotations
from typing_extensions import TYPE_CHECKING, overload

from ..constants import Uninitialized
from .basic import is_vm_function, is_vm_module, is_vm_wrapper, is_vm_callable
from .const import is_vm_const

if TYPE_CHECKING:

    from typing_extensions import TypeIs
    from ..._vm.types import VmImmutable, VmValue, VmAny


@overload
def is_vm_immutable(value: VmAny) -> TypeIs[VmImmutable]: ...
@overload
def is_vm_immutable(value, check_deep: bool) -> TypeIs[VmImmutable]: ...
def is_vm_immutable(value, check_deep=False) -> TypeIs[VmImmutable]:
    """检查值是否为 Mirascript 不可变值"""
    return (
        is_vm_module(value) or is_vm_function(value) or is_vm_const(value, check_deep)
    )


@overload
def is_vm_value(value: VmAny) -> TypeIs[VmValue]: ...
@overload
def is_vm_value(value, check_deep: bool) -> TypeIs[VmValue]: ...
def is_vm_value(value, check_deep=False) -> TypeIs[VmValue]:
    """检查值是否为 Mirascript 合法值"""
    return (
        is_vm_wrapper(value) or is_vm_function(value) or is_vm_const(value, check_deep)
    )


def is_vm_any(value, check_deep: bool) -> TypeIs[VmAny]:
    """检查值是否为 Mirascript 值（包括未初始化变量）"""
    if value is Uninitialized:
        return True
    return is_vm_value(value, check_deep)
