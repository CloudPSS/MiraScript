from typing import Union

from mirascript.vm.types.const import Uninitialized
from ....types.checker import is_vm_array, is_vm_record 
from ..._helpers import  expect_array_or_record,expect_compound

def keys(data=Uninitialized):
    expect_compound('data', data,[])
    
    
    if is_vm_array(data):
        return list(range(len(data)))
    else:
        return list(data.keys()) 
      


def values(data=Uninitialized):
    expect_array_or_record('data', data, [])
    
    
    if is_vm_array(data):
        return list(data)
    else:
        return list(data.values())


def entries(data=Uninitialized):
    expect_array_or_record('data', data, [])
    if is_vm_array(data):
        # return list(enumerate(data)) # type: ignore
        
        ret = []
        for i, v in enumerate(data):
            ret.append( {'0': i, '1': v if v is not None else None} )
        return ret
        
    ret = []
    for key, v in data.items():
        ret.append( {'0': key, '1': v if v is not None else None} )
    
    return ret
        
       

