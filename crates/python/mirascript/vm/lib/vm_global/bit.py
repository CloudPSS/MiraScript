from ...operations import ToNumber_
import math

def to_int32(x):
    x = ToNumber_(x)
    if math.isnan(x):
        return 0
    
    x = int(x)
    # print("to_int32 input:", x)  # --- DEBUG ---
    # if x & 0x80000000:
    #     print("to_int32 negative adjustment:", x)  # --- DEBUG ---
    #     x -= 0x100000000
    #     print("to_int32 negative adjustment:", x)  # --- DEBUG ---
    # print("to_int32 input:", x)  # --- DEBUG ---
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
    print("b_not input:", a,result,0xFFFFFFFF)  # --- DEBUG ---
    
    return float(result)
  
def b_xor(a, b):
    a = to_int32(a)
    b = to_int32(b)
    result = (a ^ b) & 0xFFFFFFFF
    
    if result & 0x80000000:
        result -= 0x100000000
    
    print("b_xor inputs:", a,b,result  & 0xFFFFFFFF )  # --- DEBUG ---
    
    return result
  
def shl(a, b):
    a = to_int32(a)
    b = to_int32(b)
    
    print("shl inputs:", a,b)  # --- DEBUG ---
    b &= 31  # 只保留低5位（0~31）
    a &= 0xFFFFFFFF  # 保持32位范围
    return ((a << b) | (a >> (32 - b))) & 0xFFFFFFFF

def sar(a, b):
    a = to_int32(a)
    b = to_int32(b)
    print("sar inputs:", a,b)  # --- DEBUG ---
    result = (a >> b) 
    
    
    
    return result
  
def shr(a, b):
    a = to_int32(a)
    b = to_int32(b)
    a = int(ToNumber_(a))
    b = int(ToNumber_(b))
    print("shr inputs:", a,b)  # --- DEBUG ---
    if a >= 0:
        return float(int(a) >> int(b))
    else:
        return float((int(a) + 0x100000000) >> int(b))