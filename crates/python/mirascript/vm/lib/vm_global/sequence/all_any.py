from mirascript.vm.types.const import Uninitialized
from ..._helpers import  throw_error,expect_array_or_record,expect_callable
from mirascript.vm.types.checker import is_vm_array
from ....operations import  Call_,ToBoolean_

__all__ = ['all','any']   

def all(data=Uninitialized, fn=Uninitialized):
    expect_array_or_record('data', data, None)
    expect_callable('fn', fn, data)
    
    if is_vm_array(data):
        for  i, item in enumerate(data):
            ret =Call_(fn, *(item, i, data))
            if(not ToBoolean_(ret)):
                return False
        return True
    else:
        for key, item in data.items():
            ret =Call_(fn, *(item, key, data))
            if(not ToBoolean_(ret)):
                return False
        return True

def any(data=Uninitialized, fn=Uninitialized):
    expect_array_or_record('data', data, None)
    expect_callable('fn', fn, data)
    
    if is_vm_array(data):
        for  i, item in enumerate(data):
            ret =Call_(fn, *(item, i, data))
            if(ToBoolean_(ret)):
                return True
        return False
    else:
        for key, item in data.items():
            ret =Call_(fn, *(item, key, data))
            if(ToBoolean_(ret)):
                return True
        return False
    


