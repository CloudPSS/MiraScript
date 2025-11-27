from mirascript.helpers.convert.to_number import is_decimal_number, toNumber
from mirascript.vm.helpers import Element
from mirascript.vm.lib.vm_global.math_unary import trunc
from mirascript.vm.types.checker import is_vm_array,is_vm_record
from ....operations import Call_, ToString_,Type_,is_safe_integer
from ....types import  VmValue
from ..._helpers import  expect_array_or_record,expect_compound,expect_array,expect_callable,throw_error,expect_const
from mirascript.vm.types.const import Uninitialized,VM_ARRAY_MAX_LENGTH
from functools import cmp_to_key
import math

def arr_index(index):
    idx =trunc(toNumber(index,math.nan))
    if math.isnan(idx) or idx <0:
        throw_error('Array index must be a non-negative integer', index)
    
    if idx>= VM_ARRAY_MAX_LENGTH:
        throw_error(f'Array index exceeds maximum limit of {VM_ARRAY_MAX_LENGTH}', index)
    return idx

def isArrIndex(key):
    if not isinstance(key, (int,float,bool)):
        return False
    return is_decimal_number(key) ==False and key >=0 and key <= VM_ARRAY_MAX_LENGTH

def with_inner(obj, key,key_index, value):
    
    if key_index >= len(key):
        return value
    
    
    k = key[key_index]
    result =[]
    
    if is_vm_array(obj):
        result = obj.copy()
    elif is_vm_record(obj):
        result = obj.copy()
    elif isArrIndex(k):
        result = []
    else:
        result = {}
   
    if is_vm_array(result):
        idx = int(arr_index(k))
        while idx > len(result):
            result.append(None)
        # result.append(with_inner(result[idx] if idx < len(result) else None, key, key_index + 1, value))
        # result.update
        if idx == len(result):
            result.append( with_inner(None, key, key_index + 1, value))
        if idx < len(result):
            result[idx] = with_inner(result[idx], key, key_index + 1, value)
       
    else:
        key_str = ToString_(k)
        result[key_str] = with_inner(result.get(key_str, None), key, key_index + 1, value)
    return result

def normalizeEntries(data, entries):
    if len(entries) % 2 != 0:
        raise throw_error("with_ function requires even number of arguments as key-value pairs", data)
    
    entryData = []
    
    for i in range(0, len(entries), 2):
        key = entries[i]
        expect_const('key', key, data)
        if key is None or key is Uninitialized:
            continue
        t=[]
        
        if is_vm_array(key):
            if len(key) == 0 or None in key or Uninitialized in key:
                continue
            if len(key)==1:
                key =key[0]
        
            
            
        value = entries[i+1]
        entryData.append( (key, Element(value)) )
        # entryData[ToString_(key)] = Element(value)
    
    return entryData
def with_(data=Uninitialized, *args):
    expect_array_or_record('data', data, [])
    if len(args) == 0:
        return data
    
    entryData = normalizeEntries(data, args)
    if is_vm_array(data):
        result = data.copy()
        for key, element in entryData:
            index=0
            val=None
            
            if is_vm_array(key):
                index = arr_index(key[0])
                if index <0:
                    continue
                val = with_inner(result[int(index)] if index < len(result) else None,key, 1, element)
            else:
                index = arr_index(key)
                if index <0:
                    continue
                val = element
            while index >= len(result):
                result.append(None)
            result[int(index)] = val
        return result
    else:
        
        result = data.copy()
        for key, element in entryData:
            if is_vm_array(key):
                firstKey = key[0]
                prop = ToString_(firstKey)
                val = with_inner(result.get(prop, None), key, 1, element)
            else:
                prop = ToString_(key)
                val = element
            result[prop] = val
        return result