from __future__ import annotations
from typing_extensions import TypeVar, overload

from ..._vm.types import VmAny
from ..._vm.error import VmError
from ..constants import Uninitialized

T = TypeVar("T")


@overload
def to_boolean(value: VmAny) -> bool: ...
@overload
def to_boolean(value: VmAny, fallback: T) -> bool | T: ...
def to_boolean(value: VmAny, fallback: T = Uninitialized) -> bool | T:
    if isinstance(value, bool):
        return value
    if fallback is Uninitialized:
        raise VmError(f"Cannot convert to boolean: {value}", False)
    return fallback
