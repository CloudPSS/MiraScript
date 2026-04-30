import math
from types import ModuleType

from ..types import VmModule, vm_function, VmValue
from ...helpers.constants import VM_ARRAY_MAX_LENGTH
from ._helpers import _throw_error
from .vm_global.math.round import trunc


def wrap_entry(name: str, value: VmValue, module: str):
    if callable(value):
        return vm_function(f"{module}.{name}")(value)
    else:
        return value


def create_module(name: str, lib: ModuleType) -> VmModule:
    mod = {}
    keys = (
        lib.__all__
        if hasattr(lib, "__all__") and isinstance(lib.__all__, list)
        else dir(lib)
    )
    for key in keys:
        if key.startswith("_") or key.endswith("_") or not hasattr(lib, key):
            continue
        value = getattr(lib, key)
        mod[key] = wrap_entry(key, value, name)
    return VmModule(name, mod)


def _array_len(length):
    if length is None or math.isnan(length) or length <= -1:
        _throw_error("Array length must be a non-negative integer", None)

    length = trunc(length)
    if length > VM_ARRAY_MAX_LENGTH:
        _throw_error(
            f"Array length exceeds maximum limit of {VM_ARRAY_MAX_LENGTH}", None
        )
    return int(length)
