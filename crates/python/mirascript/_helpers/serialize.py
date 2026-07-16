import math
import re

from .constants import Uninitialized
from .types import is_vm_function, is_vm_extern, is_vm_array, is_vm_module, is_vm_record

MAX_DEPTH = 100


REG_IDENTIFIER = re.compile(r"(?:_+|@+|\$+|[A-Za-z])[A-Za-z0-9_]*", re.UNICODE)
REG_ORDINAL = re.compile(
    r"(?:214748364[0-7]|21474836[0-3]\d|2147483[0-5]\d{2}|214748[0-2]\d{3}|21474[0-7]\d{4}|2147[0-3]\d{5}|214[0-6]\d{6}|21[0-3]\d{7}|20\d{8}|1\d{9}|[1-9]\d{0,8}|0)",
    re.UNICODE,
)


def serializeNil() -> str:
    return "nil"


def serializeBoolean(value: bool) -> str:
    return "true" if value else "false"


def serializeNumber(value: float) -> str:
    if math.isnan(value):
        return "nan"
    if not math.isfinite(value):
        return "-inf" if value < 0 else "inf"
    if value == 0:
        if math.copysign(1, value) < 0:
            return "-0"
        return "0"
    return str(value)


STRING_QUOTE = "'"
STRING_REG = re.compile(r"[\0-\x1f'\"`$\\\u2028\u2029]", re.UNICODE)


def serializeStringImpl(value: str, options):
    oq = options.serializeStringQuote(STRING_QUOTE, True, options)
    cq = options.serializeStringQuote(STRING_QUOTE, False, options)
    if len(value) == 0:
        return oq + cq

    if not STRING_REG.search(value):
        # 不包含特殊字符
        c = options.serializeStringContent(value, options)
        return oq + c + cq

    ret = oq
    for char in value:
        code = ord(char)
        if char == "'":
            esc = options.serializeStringEscape("\\'", "'", options)
            ret += esc
        elif char == "\0":
            esc = options.serializeStringEscape("\\0", "\0", options)
            ret += esc
        elif char == "\n":
            esc = options.serializeStringEscape("\\n", "\n", options)
            ret += esc
        elif char == "\r":
            esc = options.serializeStringEscape("\\r", "\r", options)
            ret += esc
        elif char == "\t":
            esc = options.serializeStringEscape("\\t", "\t", options)
            ret += esc
        elif char == "\b":
            esc = options.serializeStringEscape("\\b", "\b", options)
            ret += esc
        elif char == "\f":
            esc = options.serializeStringEscape("\\f", "\f", options)
            ret += esc
        elif char == "\v":
            esc = options.serializeStringEscape("\\v", "\v", options)
            ret += esc
        elif char == "\\":
            esc = options.serializeStringEscape("\\\\", "\\", options)
            ret += esc
        elif char == "$":
            esc = options.serializeStringEscape("\\$", "$", options)
            ret += esc
        elif 0xD800 <= code <= 0xDFFF:
            ret += "�"
        elif code == 0x2028 or code == 0x2029:
            ret += options.serializeStringEscape(f"\\u{{{code:x}}}", char, options)
        elif code <= 0x1F:
            ret += options.serializeStringEscape(f"\\x{code:02x}", char, options)
        else:
            ret += char
    ret += cq
    return ret


def serializeArray(value, depth, options) -> str:
    if len(value) == 0:
        return "[]"
    str_ = "["
    for i, v in enumerate(value):
        if i > 0:
            str_ += ", "
        str_ += serializeImpl(v, depth, options)
    str_ += "]"
    return str_


def serializeRecord(value, depth, options) -> str:
    custom_value = custom_value_of(value)
    if custom_value is not None:
        return serializeImpl(custom_value, depth - 1, options)
    entries = list(value.items())
    if len(entries) == 0:
        return "()"
    if len(entries) == 1:
        k, v = entries[0]
        if k == "0":
            return f"({serializeImpl(v, depth, options)},)"
        return f"({serialize_prop_name(k)}: {serializeImpl(v, depth, options)})"
    omit_key = len(entries) < 10 and all(
        str(index) == key for index, (key, _) in enumerate(entries)
    )
    str_ = "("
    for idx, (key, val) in enumerate(entries):
        if len(str_) > 1:
            str_ += ", "
        if omit_key:
            str_ += serializeImpl(val, depth, options)
        else:
            str_ += f"{serialize_prop_name(key)}: {serializeImpl(val, depth, options)}"
    str_ += ")"
    return str_


def displayFunction(value):
    try:
        name = value.__name__
        if name:
            return f"<function {name}>"
        return "<function>"
    except Exception:
        return "<function>"


class DEFAULT_OPTIONS:
    serializeNil = serializeNil
    serializeBoolean = serializeBoolean
    serializeNumber = serializeNumber
    serializeString = serializeStringImpl
    serializeStringQuote = lambda value: value
    serializeStringEscape = lambda value: value
    serializeStringContent = lambda value: value
    serializeArray = serializeArray
    serializeRecord = serializeRecord
    serializePropName = str
    serializeFunction = serializeNil
    serializeModule = serializeNil
    serializeExtern = serializeNil


# DEFAULT_OPTIONS= DefaultOptions()


def mergeOptions(base, options):
    if options is None or options is Uninitialized:
        return base
    result = base.copy()
    for key, value in options.items():
        result[key] = value
    return result


def getSerializeOptions(options):
    if options is None or options is Uninitialized:
        return DISPLAY_OPTIONS
    return mergeOptions(DISPLAY_OPTIONS, options)


def serialize_string(value: str) -> str:
    import re

    if not re.search(r"[\p{C}'\"`$\\]", value):
        return f"'{value}'"
    ret = "'"
    for char in value:
        if char == "'":
            ret += "\\'"
        elif char == "\0":
            ret += "\\0"
        elif char == "\n":
            ret += "\\n"
        elif char == "\r":
            ret += "\\r"
        elif char == "\t":
            ret += "\\t"
        elif char == "\b":
            ret += "\\b"
        elif char == "\f":
            ret += "\\f"
        elif char == "\v":
            ret += "\\v"
        elif char == "\\":
            ret += "\\\\"
        elif char == "$":
            ret += "\\$"
        elif re.match(r"\p{C}", char):
            code = ord(char)
            if code <= 0x7F:
                ret += f"\\x{code:02x}"
            elif 0xD800 <= code <= 0xDFFF:
                ret += "�"
            else:
                ret += f"\\u{{{code:x}}}"
        else:
            ret += char
    ret += "'"
    return ret


def serialize_prop_name(value: str) -> str:
    if REG_ORDINAL.fullmatch(value):
        return value
    if REG_IDENTIFIER.fullmatch(value):
        return value
    return serialize_string(value)


def custom_value_of(value):
    this_value_of = getattr(value, "valueOf", None)
    if not callable(this_value_of):
        return None
    custom_value = this_value_of()
    if custom_value is value:
        return None
    return custom_value


def serializeImpl(value, depth: int, options) -> str:
    if value is None or depth > MAX_DEPTH:
        return options.serializeNil()
    if isinstance(value, bool):
        return options.serializeBoolean(value)
    if isinstance(value, (int, float)):
        return options.serializeNumber(value)
    if isinstance(value, str):
        return options.serializeString(value, options)

    if is_vm_function(value):
        return options.serializeFunction(value)
    if is_vm_module(value):
        return options.serializeModule(value)
    if is_vm_extern(value):
        return options.serializeExtern(value)

    if is_vm_array(value):
        return serializeArray(value, depth + 1, options)
    if is_vm_record(value):
        return serializeRecord(value, depth + 1, options)

    # 不支持序列化的值
    return options.serializeNil()


def serialize(value, options=Uninitialized) -> str:
    return serializeImpl(value, 0, getSerializeOptions(options))


def displayWrapper(value, useBraces, fallback):
    try:
        return value.toString(useBraces)
    except Exception:
        return fallback


class DISPLAY_OPTIONS:
    serializeNil = serializeNil
    serializeBoolean = serializeBoolean
    serializeNumber = serializeNumber
    serializeString = serializeStringImpl
    serializeStringQuote = lambda value, open, options: value
    serializeStringEscape = lambda value, open, options: value
    serializeStringContent = lambda value, options: value
    serializeArray = serializeArray
    serializeRecord = serializeRecord
    serializePropName = str
    serializeFunction = displayFunction
    serializeModule = lambda value: displayWrapper(value, True, "<module>")
    serializeExtern = lambda value: displayWrapper(value, True, "<extern>")


def display(value, options=Uninitialized):
    opt = mergeOptions(DISPLAY_OPTIONS, options)
    return serializeImpl(value, 0, opt)
