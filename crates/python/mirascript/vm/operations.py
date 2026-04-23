import math
import re

from mirascript.helpers.convert.to_boolean import toBoolean
from mirascript.helpers.convert.to_format import toFormat
from mirascript.helpers.convert.to_number import toNumber
from mirascript.helpers.convert.to_string import toString
from mirascript.vm.error import VmError

from mirascript.vm.types.checker import is_vm_primitive
from mirascript.vm.types.module import VmModule
from mirascript.vm.types.wrapper import VmWrapper, isVmWrapper

from ..vm.types.checker import is_vm_const, is_vm_record

from .types.extern import VmExtern, is_vm_extern
from mirascript.vm.types.const import Uninitialized, getVmFunctionInfo
import unicodedata

## 标记当前值未返回的值
LoopContinue = type("LoopContinue", (), {})()
## 标记当前值为Break
LoopBreak = type("LoopBreak", (), {})()


def is_decimal_number(num):
    """判断数字是否为小数（不是整数）"""

    if isinstance(num, (int, float)):
        return isinstance(num, float) and not num.is_integer()
    return False


def is_safe_integer(n):
    """
    检查是否为安全整数（在 64 位浮点数精确表示范围内）
    类似于 JavaScript 的 Number.isSafeInteger()
    """
    if is_decimal_number(n):
        return False

    # 64 位浮点数能精确表示的最大整数范围
    MIN_SAFE_INTEGER = -(2**53) + 1
    MAX_SAFE_INTEGER = 2**53 - 1

    return MIN_SAFE_INTEGER <= n <= MAX_SAFE_INTEGER


def IsNumber_(a):
    if isinstance(a, bool):
        return False
    return isinstance(a, (int, float))


def isSame(a, b):
    if IsNumber_(a) and IsNumber_(b):
        return a == b or (math.isnan(a) and math.isnan(b))
    if a == b and type(a) is type(b):
        return True
    if isinstance(a, VmWrapper):
        return a.same(b)
    if isinstance(b, VmWrapper):
        return b.same(a)
    return False


def AssertInit_(val):
    if val is Uninitialized:
        raise VmError("Uninitialized value`", None)


def ToBoolean_(value):
    AssertInit_(value)
    return toBoolean(value)


def isVmArray(value):

    if type(value) is list:
        return True
    return False


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
    return toNumber(value)


def ToString_(val):
    AssertInit_(val)
    if val is None:
        return ""
    return toString(val, Uninitialized, False)


def overloadNumberString_(a, b):

    if IsNumber_(a) or IsNumber_(b):
        return True
    if isinstance(a, str) or isinstance(b, str):
        return False
    return True


def Add_(a, b):
    return ToNumber_(a) + ToNumber_(b)


def Sub_(a, b):
    return ToNumber_(a) - ToNumber_(b)


def Mul_(a, b):
    return ToNumber_(a) * ToNumber_(b)


def Div_(a, b):
    a = ToNumber_(a)
    b = ToNumber_(b)
    if b != 0:
        return a / b

    if a == 0:
        return math.nan
    return math.copysign(1, a) * math.copysign(1, b) * math.inf


def Mod_(a, b):

    x = ToNumber_(a)
    y = ToNumber_(b)
    """
    IEEE 754 标准的取余运算 (remainder operation)
    对应 ECMAScript 的 % 运算符
    
    规范定义：
    1. If either operand is NaN, the result is NaN.
    2. If x is ±Infinity, the result is NaN.
    3. If y is ±Infinity and x is finite, the result is x.
    4. If y is ±0, the result is NaN.
    5. If x is ±0 and y is nonzero and finite, the result is x.
    6. Otherwise, the result has the same sign as x and magnitude:
       abs(x) - floor(abs(x) / abs(y)) × abs(y)
    """
    # 1. If either operand is NaN, return NaN
    if math.isnan(x) or math.isnan(y):
        return math.nan

    # 2. If x is ±Infinity, return NaN
    if math.isinf(x):
        return math.nan

    # 3. If y is ±Infinity and x is finite, return x
    if math.isinf(y):
        return x

    # 4. If y is ±0, return NaN
    if y == 0:
        return math.nan

    # 5. If x is ±0 and y is nonzero and finite, return x (保留符号)
    if x == 0:
        return x  # 保留 +0 或 -0 的符号

    # 6. 计算取余：结果符号与 x 相同
    # Python 的 % 运算符结果符号与除数相同，不符合 IEEE 754
    # 需要使用 math.fmod，它的结果符号与被除数相同
    return math.fmod(x, y)


def Pow_(a, b):
    try:
        return math.pow(ToNumber_(a), ToNumber_(b))
    except ValueError as e:
        return math.nan


def And_(a, b):
    return ToBoolean_(a) and ToBoolean_(b)


def Or_(a, b):
    return ToBoolean_(a) or ToBoolean_(b)


def Gt_(a, b):
    if overloadNumberString_(a, b):
        return ToNumber_(a) > ToNumber_(b)
    else:
        return str(a) > str(b)


def Gte_(a, b):
    if overloadNumberString_(a, b):
        return ToNumber_(a) >= ToNumber_(b)
    else:
        return str(a) >= str(b)


def Lt_(a, b):
    if overloadNumberString_(a, b):
        return ToNumber_(a) < ToNumber_(b)
    else:
        return str(a) < str(b)


def Lte_(a, b):
    if overloadNumberString_(a, b):
        return ToNumber_(a) <= ToNumber_(b)
    else:
        return str(a) <= str(b)


def Eq_(a, b):

    AssertInit_(a)
    AssertInit_(b)

    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
        return a == b

    return isSame(a, b)


def Neq_(a, b):
    return not Eq_(a, b)


## 高级相等比较
def Aeq_(a, b):

    if overloadNumberString_(a, b):
        an = ToNumber_(a)
        bn = ToNumber_(b)
        EPS = 1e-15
        if math.isnan(an) or math.isnan(bn):
            return False
        if an == bn:
            return True
        absoluteDifference = abs(an - bn)
        if absoluteDifference < EPS:
            return True
        base = min(abs(an), abs(bn))
        return absoluteDifference < base * EPS
    else:
        as_str = ToString_(a)
        bs_str = ToString_(b)

        if as_str == bs_str:
            return True

        ai = unicodedata.normalize("NFC", as_str.lower())
        bi = unicodedata.normalize("NFC", bs_str.lower())
        return ai == bi


def Naeq_(a, b):
    return not Aeq_(a, b)


def Same_(a, b):
    AssertInit_(a)
    AssertInit_(b)
    return isSame(a, b)


def Nsame_(a, b):
    return not Same_(a, b)


def In_(a, b):
    AssertInit_(a)
    AssertInit_(b)
    if b is None:
        return False
    if isVmArray(b):
        if a is None:
            return a in b

        if is_vm_primitive(a):
            return a in b

        for item in b:
            if isSame(a, item):
                return True
        return False
    pk = ToString_(a)
    if isinstance(b, dict):

        return pk in b
    if isVmWrapper(b):
        return b.has(pk)

    return False


def Concat_(*args):
    result = ""
    for a in args:
        AssertInit_(a)
        result += toFormat(a, None)
    return result


def Pos_(a):
    # TO DO: implement Pos_
    return ToNumber_(a)
    pass


def Neg_(a):
    return -ToNumber_(a)


def Not_(a):
    return not ToBoolean_(a)


def Length_(a):
    AssertInit_(a)
    if isVmArray(a):
        return len(a)
    if isinstance(a, str):
        return len(a)
    if isinstance(a, dict):
        return len(a)
    if isinstance(a, VmWrapper) and hasattr(a, "length"):
        return a.length()
    raise TypeError(f"`Expected array, string or record, got {Type_(a)}")


def Omit_(a, b):
    AssertInit_(a)
    if a is None or not is_vm_record(a):
        return {}
    result = {}

    valueKeys = a.keys()
    omittedSet = set([ToString_(x) for x in b])
    for key in valueKeys:
        if key not in omittedSet:
            result[key] = a[key]
    return result


def Pick_(a, b):
    AssertInit_(a)
    if a is None or not is_vm_record(a):
        return {}
    result = {}
    for key in b:
        k = ToString_(key)
        if k in a:
            result[k] = a[k]

    return result


def sliceCore(a, start, end, exclusive):
    length = len(a)

    if math.isnan(start):
        start = 0
    elif start < 0:
        start = length + start
    if math.isnan(end):
        end = length
    elif end < 0:
        end = length + end
    if not math.isinf(start):
        start = math.ceil(start)
    else:
        if start > 0:
            start = length
        else:
            start = 0
    # --- DEBUG ---

    if math.isinf(end):
        if end > 0:
            end = length
        else:
            end = 0

    elif exclusive or not is_safe_integer(end):
        end = math.ceil(end)
    else:
        end = int(end + 1)
    return a[start:end]


def Slice_(a, start, end):
    AssertInit_(a)
    if not isVmArray(a):
        raise VmError(f"`Expected array, got {Type_(a)}", [])
    s = ToNumber_(start) if start is not None else 0
    e = ToNumber_(end) if end is not None else len(a) - 1
    return sliceCore(a, s, e, False)


def SliceExclusive_(a, start, end):
    AssertInit_(a)
    if not isVmArray(a):
        raise VmError(f"`Expected array, got {Type_(a)}", [])
    s = ToNumber_(start) if start is not None else 0
    e = ToNumber_(end) if end is not None else len(a)
    return sliceCore(a, s, e, True)


def Call_(func, *args):
    for a in args:
        AssertInit_(a)
    if isinstance(func, VmExtern) and hasattr(func, "callable"):
        return func.call(args)
    if callable(func):

        if len(args) == 0:
            return func()
        return func(*args)
    raise VmError(f"{type(func)}, {func} object is not callable", None)


def Type_(val):
    if val is Uninitialized or val is None:
        return "nil"
    if isinstance(val, bool):
        return "boolean"
    if isinstance(val, (int, float)):
        return "number"
    if isinstance(val, str):
        return "string"
    if isVmArray(val):
        return "array"
    if is_vm_record(val):
        return "record"
    if isinstance(val, VmExtern):
        return "extern"
    if isinstance(val, VmModule):
        return "module"
    if callable(val):
        return "function"
    return type(val).__name__


def IsBoolean_(val):
    AssertInit_(val)
    return isinstance(val, bool)


def IsString_(val):
    AssertInit_(val)
    return isinstance(val, str)


def IsRecord_(val):
    AssertInit_(val)
    return is_vm_record(val)


def IsArray_(val):
    AssertInit_(val)
    return isVmArray(val)


def AssertNonNil_(val):
    AssertInit_(val)
    if val is not None:
        return
    raise VmError("Expected non-nil value", None)


def Has_(obj, key):
    AssertInit_(obj)
    pk = ToString_(key)
    if obj is None:
        return False

    if isinstance(obj, VmWrapper):
        return obj.has(pk)
    if isinstance(obj, dict):
        return pk in obj
    if isinstance(obj, (list, tuple)):
        try:
            idx = int(float(key))
            return 0 <= idx < len(obj)
        except:
            return False
    if hasattr(obj, pk):
        return True
    return False


def Get_(obj, key):
    AssertInit_(obj)
    pk = ToString_(key)
    if obj is None:
        return None

    if isinstance(obj, VmWrapper):
        return obj.get(pk)
    if isinstance(obj, dict):
        return obj.get(pk, None)
    if isinstance(obj, (list, tuple)):

        try:
            idx = int(float(key))
            return obj[idx]
        except:
            return None
    if hasattr(obj, pk):
        return getattr(obj, pk)
    return None


def GetGlobal_(obj, key):
    pk = ToString_(key)
    r = obj.get(pk)
    return r


def Set_(obj, key, val):
    AssertInit_(obj)
    AssertInit_(val)
    pk = ToString_(key)
    if obj is None:
        return
    if not is_vm_extern(obj):
        raise VmError(f"`Expected extern object, got {Type_(obj)}", None)

    obj.set(pk, val)


def Iterable_(val):
    AssertInit_(val)
    if isVmWrapper(val):
        if hasattr(val, "keys"):
            return val.keys()
    if isinstance(val, (list, tuple)):
        return val
    if isinstance(val, dict):
        return val.keys()
    raise VmError(f"`Value is not iterable {Type_(val)}", None)


def RecordSpread_(val):
    AssertInit_(val)
    if val is None:
        return {}
    if isVmArray(val):
        result = {}
        for i in range(len(val)):
            result[str(i)] = val[i]
        return result
    if isinstance(val, dict):
        return val
    if is_vm_extern(val):
        result = {}
        for key in val.keys():
            value = val.get(key)
            if is_vm_primitive(value):
                result[key] = value
        return result

    if isinstance(val, VmWrapper) and hasattr(val, "to_dict"):
        return val.to_dict()

    raise VmError(f"`Expected record, extern or nil, got {Type_(val)}", None)


def ArraySpread_(val):
    AssertInit_(val)
    if val is None:
        return []
    if isVmArray(val):
        return val

    raise VmError(f"`Expected array, iterable extern or nil, got {Type_(val)}", None)


def Format_(val, fmt=None):
    AssertInit_(val)

    return toFormat(val, fmt)


def Range_(start, end):
    s = ToNumber_(start)
    e = ToNumber_(end)
    if math.isnan(s) or math.isnan(e):
        return []
    result = []
    while s <= e:
        result.append(s)
        s += 1
    return result


def RangeExclusive_(start, end):
    pass
    s = ToNumber_(start)
    e = ToNumber_(end)
    if math.isnan(s) or math.isnan(e):
        return []
    # return list(range(int(s),int(e)))
    result = []
    while s < e:
        result.append(s)
        s += 1
    return result
