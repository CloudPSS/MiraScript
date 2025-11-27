import math
import re
from mirascript.vm.error import VmError
from mirascript.vm.types.const import Uninitialized

def is_decimal_number(num):
    """判断数字是否为小数（不是整数）"""
    
    if isinstance(num, (int, float)):
        return isinstance(num, float) and not num.is_integer()
    return False
def is_number_regex(s):
    # 匹配整数、小数、科学计数法
    pattern = r'^[-+]?[0-9]*\.?[0-9]+([eE][-+]?[0-9]+)?$'
    return bool(re.match(pattern, s))
def toNumber(value,fallback=Uninitialized):
    if isinstance(value, bool):
        return float(1 if value else 0)
      
    if isinstance(value, (int, float)):
        return float(value)
    
    if isinstance(value, str): 
        value = value.strip()
        if value != "":
            try:
                if '_' in value:
                    if fallback is Uninitialized:
                        raise VmError(f"Cannot convert to number: {value}",math.nan)
                    return fallback
                if value.startswith(("0x","0X")):
                    return float(int(value,16))
                if not is_number_regex(value):
                    if value=='inf' or value=='+inf' or value=='Infinity' or value=='+Infinity':
                        return math.inf
                    if value=='-inf' or value=='-Infinity':
                        return -math.inf
                    
                    if value =='nan' or value=='NaN':
                        return math.nan
                    if fallback is Uninitialized:
                        raise VmError(f"Cannot convert to number: {value}",math.nan)
                    return fallback
                
                return float(value)
            except ValueError as e:
                pass
    
    if fallback is Uninitialized:
        raise VmError(f"Cannot convert to number: {value}",math.nan)
    
    return fallback