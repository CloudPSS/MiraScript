import math
import unicodedata

from ..._helpers.convert.to_format import toFormat
from .common import AssertInit
from .convert import ToNumber, ToBoolean, ToString
from .utils import is_number, is_same, overload_number_string

# String operations


def Concat(*args) -> str:
    result = ""
    for a in args:
        AssertInit(a)
        result += toFormat(a, None)
    return result


# Unary operations


def Pos(a) -> float:
    return ToNumber(a)


def Neg(a) -> float:
    return -ToNumber(a)


def Not(a) -> bool:
    return not ToBoolean(a)


# Math operations


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


# Logical operations without short-circuiting


def And(a, b) -> bool:
    return ToBoolean(a) and ToBoolean(b)


def Or(a, b) -> bool:
    return ToBoolean(a) or ToBoolean(b)


# Comparison operations


def Gt(a, b) -> bool:
    if overload_number_string(a, b):
        return ToNumber(a) > ToNumber(b)
    else:
        return str(a) > str(b)


def Gte(a, b) -> bool:
    if overload_number_string(a, b):
        return ToNumber(a) >= ToNumber(b)
    else:
        return str(a) >= str(b)


def Lt(a, b) -> bool:
    if overload_number_string(a, b):
        return ToNumber(a) < ToNumber(b)
    else:
        return str(a) < str(b)


def Lte(a, b) -> bool:
    if overload_number_string(a, b):
        return ToNumber(a) <= ToNumber(b)
    else:
        return str(a) <= str(b)


def Eq(a, b) -> bool:
    AssertInit(a)
    AssertInit(b)

    if is_number(a) and is_number(b):
        return a == b

    return is_same(a, b)


def Neq(a, b) -> bool:
    return not Eq(a, b)


def Aeq(a, b) -> bool:
    if overload_number_string(a, b):
        a_n = ToNumber(a)
        b_n = ToNumber(b)
        EPS = 1e-15
        if math.isnan(a_n) or math.isnan(b_n):
            return False
        if a_n == b_n:
            return True
        absoluteDifference = abs(a_n - b_n)
        if absoluteDifference < EPS:
            return True
        base = min(abs(a_n), abs(b_n))
        return absoluteDifference < base * EPS
    else:
        a_s = ToString(a)
        b_s = ToString(b)

        if a_s == b_s:
            return True

        a_i = a_s.lower()
        b_i = b_s.lower()

        if a_i == b_i:
            return True

        ai = unicodedata.normalize("NFC", a_i)
        bi = unicodedata.normalize("NFC", b_i)
        return ai == bi


def Naeq(a, b) -> bool:
    return not Aeq(a, b)


def Same(a, b) -> bool:
    AssertInit(a)
    AssertInit(b)
    return is_same(a, b)


def Nsame(a, b) -> bool:
    return not Same(a, b)
