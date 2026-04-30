import math
import unicodedata
from enum import Enum
from typing_extensions import Callable, Mapping, Any

from ..helpers.convert.to_boolean import toBoolean
from ..helpers.convert.to_format import toFormat
from ..helpers.convert.to_number import toNumber
from ..helpers.checker import is_safe_integer
from ..helpers.convert.to_string import toString
from ..helpers.constants import kVmScript
from ..helpers.types import (
    is_vm_primitive,
    is_vm_record,
    is_vm_extern,
    is_vm_array,
    is_vm_module,
    is_vm_wrapper,
)
from .error import VmError
from .types.module import VmModule
from .types.function import vm_function
from .types.types import Uninitialized, VmValue


class LoopControl(Enum):
    Continue = object()
    Break = object()


## 标记当前值未返回的值
LoopContinue = LoopControl.Continue
## 标记当前值为Break
LoopBreak = LoopControl.Break


def IsNumber(a):
    if isinstance(a, bool):
        return False
    return isinstance(a, (int, float))


def isSame(a, b) -> bool:
    if IsNumber(a) and IsNumber(b):
        return a == b or (math.isnan(a) and math.isnan(b))
    if a == b and type(a) is type(b):
        return True
    if is_vm_wrapper(a):
        return a.same(b)
    if is_vm_wrapper(b):
        return b.same(a)
    return False


def AssertInit(val):
    if val is Uninitialized:
        raise VmError("Uninitialized value`", None)


def ToBoolean(value):
    AssertInit(value)
    return toBoolean(value)


def ToNumber(value):
    """
    将输入值转换为数字

    参数:
        value: 要转换的值
        default: 转换失败时返回的默认值
        prefer_float: 是否优先转换为浮点数

    返回:
        转换后的数字，如果无法转换则返回默认值
    """
    AssertInit(value)
    return toNumber(value)


def ToString(val) -> str:
    AssertInit(val)
    if val is None:
        return ""
    return toString(val)


def _overloadNumberString(a, b):
    if IsNumber(a) or IsNumber(b):
        return True
    if isinstance(a, str) or isinstance(b, str):
        return False
    return True


def Add(a, b) -> float:
    return ToNumber(a) + ToNumber(b)


def Sub(a, b) -> float:
    return ToNumber(a) - ToNumber(b)


def Mul(a, b) -> float:
    return ToNumber(a) * ToNumber(b)


def Div(a, b) -> float:
    a = ToNumber(a)
    b = ToNumber(b)
    if b != 0:
        return a / b

    if a == 0:
        return math.nan
    return math.copysign(1, a) * math.copysign(1, b) * math.inf


def Mod(a, b) -> float:
    x = ToNumber(a)
    y = ToNumber(b)
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


def Pow(a, b) -> float:
    try:
        return math.pow(ToNumber(a), ToNumber(b))
    except ValueError:
        return math.nan


def And(a, b) -> bool:
    return ToBoolean(a) and ToBoolean(b)


def Or(a, b) -> bool:
    return ToBoolean(a) or ToBoolean(b)


def Gt(a, b) -> bool:
    if _overloadNumberString(a, b):
        return ToNumber(a) > ToNumber(b)
    else:
        return str(a) > str(b)


def Gte(a, b) -> bool:
    if _overloadNumberString(a, b):
        return ToNumber(a) >= ToNumber(b)
    else:
        return str(a) >= str(b)


def Lt(a, b) -> bool:
    if _overloadNumberString(a, b):
        return ToNumber(a) < ToNumber(b)
    else:
        return str(a) < str(b)


def Lte(a, b) -> bool:
    if _overloadNumberString(a, b):
        return ToNumber(a) <= ToNumber(b)
    else:
        return str(a) <= str(b)


def Eq(a, b) -> bool:
    AssertInit(a)
    AssertInit(b)

    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
        return a == b

    return isSame(a, b)


def Neq(a, b) -> bool:
    return not Eq(a, b)


## 高级相等比较
def Aeq(a, b) -> bool:
    if _overloadNumberString(a, b):
        an = ToNumber(a)
        bn = ToNumber(b)
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
        as_str = ToString(a)
        bs_str = ToString(b)

        if as_str == bs_str:
            return True

        ai = unicodedata.normalize("NFC", as_str.lower())
        bi = unicodedata.normalize("NFC", bs_str.lower())
        return ai == bi


def Naeq(a, b) -> bool:
    return not Aeq(a, b)


def Same(a, b) -> bool:
    AssertInit(a)
    AssertInit(b)
    return isSame(a, b)


def Nsame(a, b) -> bool:
    return not Same(a, b)


def In(a, b) -> bool:
    AssertInit(a)
    AssertInit(b)
    if b is None:
        return False
    if is_vm_array(b):
        if a is None:
            return a in b

        if is_vm_primitive(a):
            return a in b

        for item in b:
            if isSame(a, item):
                return True
        return False
    pk = ToString(a)
    if isinstance(b, dict):
        return pk in b
    if is_vm_wrapper(b):
        return b.has(pk)

    return False


def Concat(*args) -> str:
    result = ""
    for a in args:
        AssertInit(a)
        result += toFormat(a, None)
    return result


def Pos(a) -> float:
    return ToNumber(a)


def Neg(a) -> float:
    return -ToNumber(a)


def Not(a) -> bool:
    return not ToBoolean(a)


def Length(a) -> float:
    AssertInit(a)
    if isinstance(a, (str, list, dict)):
        return float(len(a))
    raise TypeError(f"`Expected array, string or record, got {Type(a)}")


def Omit(a, b):
    AssertInit(a)
    if a is None or not is_vm_record(a):
        return {}
    result = {}

    valueKeys = a.keys()
    omittedSet = set([ToString(x) for x in b])
    for key in valueKeys:
        if key not in omittedSet:
            result[key] = a[key]
    return result


def Pick(a, b):
    AssertInit(a)
    if a is None or not is_vm_record(a):
        return {}
    result = {}
    for key in b:
        k = ToString(key)
        if k in a:
            result[k] = a[k]

    return result


def _slice(a, start, end, exclusive):
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


def Slice(a, start, end):
    AssertInit(a)
    if not is_vm_array(a):
        raise VmError(f"`Expected array, got {Type(a)}", [])
    s = ToNumber(start) if start is not None else 0
    e = ToNumber(end) if end is not None else len(a) - 1
    return _slice(a, s, e, False)


def SliceExclusive(a, start, end):
    AssertInit(a)
    if not is_vm_array(a):
        raise VmError(f"`Expected array, got {Type(a)}", [])
    s = ToNumber(start) if start is not None else 0
    e = ToNumber(end) if end is not None else len(a)
    return _slice(a, s, e, True)


def Call(func, *args):
    for a in args:
        AssertInit(a)
    # if isinstance(func, VmExtern) and hasattr(func, "callable"):
    #     return func.call(args)
    if callable(func):
        if len(args) == 0:
            return func()
        return func(*args)
    raise VmError(f"{type(func)}, {func} object is not callable", None)


def Type(val):
    if val is Uninitialized or val is None:
        return "nil"
    if isinstance(val, bool):
        return "boolean"
    if isinstance(val, (int, float)):
        return "number"
    if isinstance(val, str):
        return "string"
    if is_vm_array(val):
        return "array"
    if is_vm_record(val):
        return "record"
    if is_vm_extern(val):
        return "extern"
    if is_vm_module(val):
        return "module"
    if callable(val):
        return "function"
    return type(val).__name__


def IsBoolean(val):
    AssertInit(val)
    return isinstance(val, bool)


def IsString(val):
    AssertInit(val)
    return isinstance(val, str)


def IsRecord(val):
    AssertInit(val)
    return is_vm_record(val)


def IsArray(val):
    AssertInit(val)
    return is_vm_array(val)


def AssertNonNil(val):
    AssertInit(val)
    if val is not None:
        return
    raise VmError("Expected non-nil value", None)


def Has(obj, key):
    AssertInit(obj)
    pk = ToString(key)
    if obj is None:
        return False

    if is_vm_wrapper(obj):
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


def Get(obj, key):
    AssertInit(obj)
    pk = ToString(key)
    if obj is None:
        return None

    if is_vm_wrapper(obj):
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


def Set(obj, key, val):
    AssertInit(obj)
    AssertInit(val)
    pk = ToString(key)
    if obj is None:
        return
    if not is_vm_extern(obj):
        raise VmError(f"`Expected extern object, got {Type(obj)}", None)

    obj.set(pk, val)


def Iterable(val):
    AssertInit(val)
    if is_vm_wrapper(val):
        if hasattr(val, "keys"):
            return val.keys()
    if isinstance(val, (list, tuple)):
        return val
    if isinstance(val, dict):
        return val.keys()
    raise VmError(f"`Value is not iterable {Type(val)}", None)


def RecordSpread(val):
    AssertInit(val)
    if val is None:
        return {}
    if is_vm_record(val):
        return val
    if is_vm_array(val):
        return {ToString(i): val[i] for i in range(len(val))}
    if is_vm_extern(val):
        return {}

    raise VmError(f"`Expected record, extern or nil, got {Type(val)}", None)


def ArraySpread(val):
    AssertInit(val)
    if val is None:
        return []
    if is_vm_array(val):
        return val

    raise VmError(f"`Expected array, iterable extern or nil, got {Type(val)}", None)


def Format(val, fmt=None):
    AssertInit(val)

    return toFormat(val, fmt)


def Range(start, end):
    s = ToNumber(start)
    e = ToNumber(end)
    if math.isnan(s) or math.isnan(e):
        return []
    result = []
    while s <= e:
        result.append(s)
        s += 1
    return result


def RangeExclusive(start, end):
    pass
    s = ToNumber(start)
    e = ToNumber(end)
    if math.isnan(s) or math.isnan(e):
        return []
    result = []
    while s < e:
        result.append(s)
        s += 1
    return result


def Fn(name):

    def decorator(func):

        from .helpers import CpEnter, CpExit

        fn = func

        def fn_wrapper(*args, **kwargs):
            try:
                CpEnter()
                return fn(*args, **kwargs)
            finally:
                CpExit()

        return vm_function(name)(fn_wrapper)

    return decorator


def Closure(func):

    from .helpers import CpEnter, CpExit

    def closure_wrapper():
        try:
            CpEnter()
            return func()
        finally:
            CpExit()

    return closure_wrapper


def Script(func):

    from mirascript.vm.helpers import CpEnter, CpExit

    def script_wrapper(*args, **kwargs):
        try:
            CpEnter()
            return func(*args, **kwargs)
        finally:
            CpExit()

    setattr(script_wrapper, kVmScript, True)
    return script_wrapper


_PUB_ATTR = "__mirascript.mod.pub__"


def Module(name: str):

    from .helpers import Upvalue

    def decorator(kls: type):

        pub: "dict[str, Callable[[], Any]]" = {}
        for attr_name in dir(kls):
            attr = getattr(kls, attr_name)
            pub_name = getattr(attr, _PUB_ATTR, None)
            if pub_name is not None:
                assert callable(attr), f"Public attribute {attr_name} must be callable"
                pub[pub_name] = attr

        class Mod(Mapping[str, VmValue]):

            def __getitem__(self, key: str) -> VmValue:
                getter = pub[key]
                return Upvalue(getter())

            def __iter__(self):
                return iter(pub.keys())

            def __len__(self):
                return len(pub)

        return VmModule(name, Mod())

    return decorator


def Pub(name: str):

    def decorator(method: Callable):
        setattr(method, _PUB_ATTR, name)
        return method

    return decorator
