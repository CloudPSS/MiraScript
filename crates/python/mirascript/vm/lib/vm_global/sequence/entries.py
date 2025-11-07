from typing import Union
from ....types.checker import is_vm_array, is_vm_record 
from ..._helpers import VmLib, expect_array_or_record,expect_compound

def keys(*args):
    if len(args) == 0:
        raise TypeError('keys requires at least one argument')
    data= args[0]
    expect_compound('data', data,data)
    if is_vm_array(data):
        return list(range(len(data)))
    else:
        return list(data.keys()) 
      


def values(*args):
    if len(args) == 0:
        raise TypeError('values requires at least one argument')
    data= args[0]
    expect_array_or_record('data', data, data)
    if is_vm_array(data):
        return list(data)
    else:
        return list(data.values())


def entries(*args):
    if len(args) == 0:
        raise TypeError('entries requires at least one argument')
    data= args[0]
    expect_array_or_record('data', data, data)
    if is_vm_array(data):
        return list(enumerate(data)) # type: ignore
    else:
        return list(data.items()) # type: ignore

