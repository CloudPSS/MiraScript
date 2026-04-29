from typing_extensions import TypeVar, overload

from mirascript.vm.error import VmError
from mirascript.vm.types.types import Uninitialized

T = TypeVar("T")


@overload
def toBoolean(value) -> bool: ...
@overload
def toBoolean(value, fallback: T) -> "bool | T": ...
def toBoolean(value, fallback: T = Uninitialized) -> "bool | T":
    if isinstance(value, bool):
        return value
    if fallback is Uninitialized:
        raise VmError(f"Cannot convert to boolean: {value}", False)
    return fallback
