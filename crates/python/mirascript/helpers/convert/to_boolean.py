from mirascript.vm.error import VmError
from mirascript.vm.types.const import Uninitialized


def toBoolean(value, fallback=Uninitialized):
    if isinstance(value, bool):
        return value
    if fallback is Uninitialized:
        raise VmError(f"Cannot convert to boolean: {value}", False)
    return fallback
