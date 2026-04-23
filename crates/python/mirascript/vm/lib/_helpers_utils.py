import math
from ._helpers import  throw_error
from ..types import VmFunction, VmModule
from mirascript.vm.lib.vm_global.math_unary import trunc
from ..types.const import VM_ARRAY_MAX_LENGTH, VmAny, VmValue
from ..error import VmError



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
        mod[key] = wrap_entry(key, value,name)
    return VmModule(name, mod)


def array_len(length):
    if length is None or math.isnan(length) or length<=-1:
        throw_error('Array length must be a non-negative integer', None)
        
    # length = int(length)
    length =trunc(length)
    if length > VM_ARRAY_MAX_LENGTH:
        throw_error(f"Array length exceeds maximum limit of {VM_ARRAY_MAX_LENGTH}", None)
    return int(length)