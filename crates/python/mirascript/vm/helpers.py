import time
import math

from mirascript.vm.types.const import VM_ARRAY_MAX_LENGTH
from .operations import ToNumber_, ToBoolean_, isNumber_,AssertInit_
from .types.checker import  is_vm_const
from .types.context import DefaultVmContext, VmSharedContext
def Vargs(varags):
    """将非常量元素置为 None，返回新列表"""
    # 这里假设 is_vm_const 已在其他地方实现
    args = list(varags)
    for i in range(len(args)):
        el = args[i]
        if not is_vm_const(el):
            args[i] = None
    return args

def Element(value):
    # 假设 $AssertInit 已在其他地方实现
    AssertInit_(value)
    if not is_vm_const(value):
        return None
    return value

def ElementOpt(key, value):
    AssertInit_(value)
    print('ElementOpt called with key:', key, 'value:', value)  # --- DEBUG ---
    if value is None or not is_vm_const(value):
        return {}
    return {key: value}



def Upvalue(value):
    AssertInit_(value)
    return value

def assert_array_length(start, end):
    if end - start > VM_ARRAY_MAX_LENGTH:
        raise RuntimeError(f"Array length exceeds maximum limit of {VM_ARRAY_MAX_LENGTH}")

def is_empty_range(start, end):
    return not math.isfinite(start) or not math.isfinite(end) or start > end

def ArrayRange(start, end):
    s = ToNumber_(start)
    e = ToNumber_(end)
    if is_empty_range(s, e):
        return []
    assert_array_length(s, e)
    arr = []
    i =s
    while i <= e:
        arr.append(i)
        i += 1
    return arr

def ArrayRangeExclusive(start, end):
    s = ToNumber_(start)
    e = ToNumber_(end)
    if is_empty_range(s, e):
        return []
    assert_array_length(s, e)
    arr = []
    i = s
    while i < e:
        arr.append(i)
        i += 1
    return arr





MAX_DEPTH = 128
cp_depth = 0
cp = float('nan')
cp_timeout = 100  # 默认超时时间，单位毫秒

def Cp():
    """检查点"""
    pass
    global cp
    current_time = int(time.time() * 1000)
    if not cp or (isinstance(cp, float) and cp != cp):  # NaN 检查
        cp = current_time
    elif current_time - cp > cp_timeout:
        raise RuntimeError(f'Execution timeout, exceeded {cp_timeout} ms , last checkpoint at {cp} ms , current time {current_time} ms')

def CpEnter():
    """进入检查点"""
    global cp_depth, cp
    cp_depth += 1
    if cp_depth <= 1:
        cp = int(time.time() * 1000)
        cp_depth = 1
    elif cp_depth > MAX_DEPTH:
        raise RuntimeError('Maximum call depth exceeded')
    else:
        Cp()

def CpExit():
    """退出检查点"""
    global cp_depth, cp
    cp_depth -= 1
    if cp_depth < 1:
        cp = float('nan')
        cp_depth = 0
    else:
        Cp()

def config_checkpoint(timeout=100):
    """设置检查点超时时间"""
    global cp_timeout
    if not isinstance(timeout, (int, float)) or timeout is None or timeout <= 0 or (isinstance(timeout, float) and timeout != timeout):
        raise ValueError('Invalid timeout value')
    cp_timeout = timeout
 
       
        
        
def GlobalFallback():
    return DefaultVmContext