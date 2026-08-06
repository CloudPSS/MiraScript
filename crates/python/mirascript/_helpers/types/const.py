from __future__ import annotations
from typing_extensions import TYPE_CHECKING, overload, Final

from .basic import (
    is_vm_array,
    is_vm_function,
    is_vm_record,
    is_vm_wrapper,
    is_vm_primitive,
)

if TYPE_CHECKING:

    from typing_extensions import Literal, TypeIs
    from ..._vm.types import VmConst, VmAny, VmArray, VmRecord

_MAX_DEPTH: Final = 16


def _is_vm_array_deep(value: VmArray, depth: int) -> bool:
    """检查数组是否为 Mirascript 常量"""
    if depth <= 0:
        return True
    inner_depth = depth - 1
    return all(_is_vm_const_inner(v, inner_depth) for v in value)


def _is_vm_record_deep(value: VmRecord, depth: int) -> bool:
    """检查记录是否为 Mirascript 常量"""
    if depth <= 0:
        return True
    inner_depth = depth - 1
    return all(
        _is_vm_const_inner(v, inner_depth) and type(k) is str for k, v in value.items()
    )


def _is_vm_const_inner(value, depth: int) -> TypeIs[VmConst]:
    """检查值是否为 Mirascript 常量"""
    if is_vm_function(value) or is_vm_wrapper(value):
        return False
    if is_vm_primitive(value):
        return True
    if is_vm_array(value):
        return _is_vm_array_deep(value, depth)
    if is_vm_record(value):
        return _is_vm_record_deep(value, depth)
    return False


@overload
def is_vm_const(
    value: VmAny, check_deep: Literal[False] = False
) -> TypeIs[VmConst]: ...
@overload
def is_vm_const(value, check_deep: Literal[True]) -> TypeIs[VmConst]: ...
def is_vm_const(value, check_deep=False):
    """检查值是否为 Mirascript 常量"""
    return _is_vm_const_inner(value, _MAX_DEPTH if check_deep else 0)
