import math

from vm.VmWrapper import VmWrapper

Uninitialized = type("Uninitialized", (), {})()

def isNumber_(a):
    return isinstance(a, (int, float))

def isSame(a,b):
    if(isNumber_(a) and isNumber_(b)):
        return a ==b or (math.isnan(a) and math.isnan(b))
    
    if a==b :
        return True
    if isinstance(a,VmWrapper):
        return a.same(b)
    if isinstance(b,VmWrapper):
        return b.same(a) 
    return False
def AssertInit_(val):
  if val is Uninitialized:
    raise Exception("Uninitialized value`")

def ToBoolean_(value):
    AssertInit_(value)
    
    return value is not None and value!=False
    
def isVmArray(value):
    if type(value) is list:
        return True

def ToNumber_(value):
    """
    将输入值转换为数字
    
    参数:
        value: 要转换的值
        default: 转换失败时返回的默认值
        prefer_float: 是否优先转换为浮点数
    
    返回:
        转换后的数字，如果无法转换则返回默认值
    """
    AssertInit_(value)
    
    if isinstance(value, (int, float)):
        return value
    
    if not isinstance(value, str):
        try:
            value = str(value)
        except:
            raise ValueError("")
    
    value = value.strip()
    if not value:
        raise ValueError("")
    
    try:
        if '.' in value or 'e' in value.lower():
            return float(value)
        return int(value)
    except ValueError:
        raise ValueError("")

def ToString_(val):
    AssertInit_(val)
    if val is None:
        return ""
    

def overloadNumberString_(a,b):
    
    return isNumber_(a) or isNumber_(b) or type(a) is not str or type(b) is not str




def Add_(a,b):
  
  return ToNumber_(a)+ToNumber_(b)

def Sub_(a,b):
    return ToNumber_(a)-ToNumber_(b)

def Mul_(a,b):
    return ToNumber_(a)*ToNumber_(b)

def Div_(a,b):
    return ToNumber_(a)/ToNumber_(b)

def Mod_(a,b):
    return ToNumber_(a)%ToNumber_(b)

def Pow_(a,b):
    return math.pow(ToNumber_(a),ToNumber_(b))

def And_(a,b):
    return ToBoolean_(a) and ToBoolean_(b)

def Or_(a,b):
    return ToBoolean_(a) or ToBoolean_(b)

def Gt_(a,b):
    if overloadNumberString_(a,b):
        return ToNumber_(a) > ToNumber_(b)
    else:
        return str(a) >str(b)
    
def Gte_(a,b):
    if overloadNumberString_(a,b):
        return ToNumber_(a) >= ToNumber_(b)
    else:
        return str(a) >= str(b)
    
def Lt_(a,b):
    if overloadNumberString_(a,b):
        return ToNumber_(a) < ToNumber_(b)
    else:
        return str(a) < str(b)
    
def Lte_(a,b):
    if overloadNumberString_(a,b):
        return ToNumber_(a) <= ToNumber_(b)
    else:
        return str(a) <= str(b)

def Eq_(a,b):
    
    AssertInit_(a)
    AssertInit_(b)
    
    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
        return a==b

    return isSame(a,b)

def Neq_(a,b):
    return not Eq_(a,b)

    

def Aeq_(a,b):
    pass

def Naeq_(a,b):
    return not Aeq_(a,b)

def Same_(a,b):
    AssertInit_(a)
    AssertInit_(b)
    return isSame(a,b)

def in_(a,b):
    AssertInit_(a)
    AssertInit_(b)
    return a in b

def Concat_(*args):
    pass


def Pos_(a):
    pass

def Neg_(a):
    pass

def Not_(a):
    pass

def Length_(a):
    return

def Omit_(a,b):
    pass

def Pick_(a,b):
    pass

def Slice_(a,start,end):
    pass

def SliceExclusive_(a,start,end):
    pass



def Call_(func,args):
    pass

def Type_(val):
    pass

def IsBoolean_(val):
    pass

def IsString_(val):
    pass

def IsRecord_(val):
    pass

def IsArray_(val):
    pass

def AssertNonNil_(val):
    pass

def Has_(obj,key):
    pass

def Get_(obj,key,defval):
    pass

def Set_(obj,key,val):
    pass

def Iterable_(val):
    pass

def RecordSpread_(val):
    pass

def ArraySpread_(val):
    pass