import math
from ..types import VmFunction, VmModule
from ..types.const import VM_ARRAY_MAX_LENGTH
from ._helpers import _throw_error
from .vm_global.math.round import trunc


def wrap_entry(name: str, value, module: str):
    if callable(value):
        # Python 函数名不可直接更改，跳过重命名
        return VmFunction(
            value,
            {
                "isLib": True,
                "injectCp": True,
                "fullName": f"{module}.{name}",
            },
        )
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
