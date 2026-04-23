from mirascript.vm.lib._helpers import expect_number
import math

def to_int32(x):
    x = expect_number('x', x)
    if math.isnan(x):
        return 0
    
    x = int(x)
    return x 

def b_and(a, b):
    a = to_int32(a)
    b = to_int32(b)

    
    result =( a & b) 
    
    if result & 0x80000000:
        result -= 0x100000000
    
    
    return float(result)
  
def b_or(a, b):
    a = to_int32(a)
    b = to_int32(b)
    result = (a | b) & 0xFFFFFFFF
    if result & 0x80000000:
        result -= 0x100000000
    return float(result)

def b_not(a):
    a = to_int32(a)
    result = (~a) & 0xFFFFFFFF
    if result & 0x80000000:
        result -= 0x100000000
    
    return float(result)
  
def b_xor(a, b):
    a = to_int32(a)
    b = to_int32(b)
    result = (a ^ b) & 0xFFFFFFFF
    
    if result & 0x80000000:
        result -= 0x100000000
    
    
    return result
  
def shl(a, b):
    a = to_int32(a)
    b = to_int32(b)
    
    b &= 31  # 只保留低5位（0~31）
    a &= 0xFFFFFFFF  # 保持32位范围
    return ((a << b) | (a >> (32 - b))) & 0xFFFFFFFF

def sar(a, b):
    a = to_int32(a)
    b = to_int32(b)
    result = (a >> b) 
    
    
    
    return result
  
def shr(a, b):
    a = to_int32(a)
    b = to_int32(b)
    a = int(expect_number('a', a))
    b = int(expect_number('b', b))
    if a >= 0:
        return float(int(a) >> int(b))
    else:
        return float((int(a) + 0x100000000) >> int(b))