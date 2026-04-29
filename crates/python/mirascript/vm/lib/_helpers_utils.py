import math
from ..types import VmModule, vm_function, VmValue
from ...helpers.constants import VM_ARRAY_MAX_LENGTH
from ._helpers import _throw_error
from .vm_global.math.round import trunc


def wrap_entry(name: str, value: VmValue, module: str):
    if callable(value):
        return vm_function(f"{module}.{name}")(value)
    else:
        return value


def create_module(name: str, lib: dict) -> VmModule:
    mod = {}
    for key, value in lib.items():
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
