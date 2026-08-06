from __future__ import annotations
from typing_extensions import TYPE_CHECKING, overload

from ..constants import Uninitialized
from .basic import is_vm_record, is_vm_array, is_vm_function, is_vm_wrapper

if TYPE_CHECKING:

    from typing_extensions import Literal
    from ..._vm.types import VmValue, VmAny, VmTypeName


@overload
def get_vm_type(value: VmValue) -> VmTypeName: ...
@overload
def get_vm_type(value: VmAny) -> Literal["uninitialized"] | VmTypeName: ...


def get_vm_type(value: VmAny) -> Literal["uninitialized"] | VmTypeName:
    """获取 Mirascript 类型"""
    if value is Uninitialized:
        return "uninitialized"
    if value is None:
        return "nil"
    t = type(value)
    if t is bool:
        return "boolean"
    if t in (int, float):
        return "number"
    if t is str:
        return "string"
    if is_vm_record(value):
        return "record"
    if is_vm_array(value):
        return "array"
    if is_vm_function(value):
        return "function"
    if is_vm_wrapper(value):
        return value.type
    raise TypeError(f"Unknown Mirascript type: {type(value)}")
