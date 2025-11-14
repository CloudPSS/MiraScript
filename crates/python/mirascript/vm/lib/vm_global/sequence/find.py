from ..._helpers import  VmLibOption, expect_array_or_record, expect_callable,required
from ....types.checker import is_vm_array, is_vm_callable
from ....operations import  Call_,ToBoolean_,Same_
from mirascript.vm.types.const import Uninitialized
def find_(data=Uninitialized, predicate=Uninitialized):
    expect_array_or_record('data', data, None)
    required('predicate', predicate, None)
    if is_vm_callable(predicate) == False:
        def p(value, key, data):
            return Same_( value, predicate)
    else:
        def p(value, key, data):
            # ret = predicate(value, key, data)
            ret = Call_(predicate, *(value, key, data))
            return ToBoolean_(ret)
    
    if is_vm_array(data):
        for i, item in enumerate(data):
            value = item if item is not None else None
            if p(value, i, data):
                return {'0': i, '1': value}
        return None
    else:
        for key, v in data.items():
            value = v if v is not None else None
            if p(value, key, data):
                return {'0': key, '1': value}
        return None

