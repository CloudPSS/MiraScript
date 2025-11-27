import math

from mirascript.helpers.types import isVmFunction
from mirascript.vm.types.checker import is_vm_array, is_vm_module, is_vm_record
from mirascript.vm.types.const import Uninitialized, getVmFunctionInfo
from mirascript.vm.types.extern import is_vm_extern
from .constants import REG_IDENTIFIER, REG_ORDINAL

MAX_DEPTH = 100

REG_IDENTIFIER_FULL = REG_IDENTIFIER
REG_ORDINAL_FULL = REG_ORDINAL

def serializeNil() -> str:
    return 'nil'

def serializeBoolean(value: bool) -> str:
    return 'true' if value else 'false'

def serializeNumber(value: float) -> str:
    if math.isnan(value):
        return 'nan'
    if not math.isfinite(value):
        return '-inf' if value < 0 else 'inf'
    if value ==0:
        if math.copysign(1,value)<0:
            return '-0'
        return '0'
    return str(value)



def serializeStringImpl(value,options):
    import re
    if not re.search(r'[\\p{C}\'"`$\\]', value, re.UNICODE):
        # 不包含特殊字符
        oq = options.serializeStringQuote("'", True, options)
        cq = options.serializeStringQuote("'", False, options)
        c = options.serializeStringContent(value, options)
        return oq + c + cq
    ret = options.serializeStringQuote("'", True, options)
    for char in value:
        if char == "'":
            esc = options.serializeStringEscape("\\'", "'", options)
            ret += esc
        elif char == '\0':
            esc = options.serializeStringEscape("\\0", '\0', options)
            ret += esc
        elif char == '\n':
            esc = options.serializeStringEscape("\\n", '\n', options)
            ret += esc
        elif char == '\r':
            esc = options.serializeStringEscape("\\r", '\r', options)
            ret += esc
        elif char == '\t':
            esc = options.serializeStringEscape("\\t", '\t', options)
            ret += esc
        elif char == '\b':
            esc = options.serializeStringEscape("\\b", '\b', options)
            ret += esc
        elif char == '\f':
            esc = options.serializeStringEscape("\\f", '\f', options)
            ret += esc
        elif char == '\v':
            esc = options.serializeStringEscape("\\v", '\v', options)
            ret += esc
        elif char == '\\':
            esc = options.serializeStringEscape("\\\\", '\\', options)
            ret += esc
        elif char == '$':
            esc = options.serializeStringEscape("\\$", '$', options)
            ret += esc
        elif re.match(r'\p{C}', char, re.UNICODE):
            code = ord(char)
            if code <= 0x7f:
                esc = options.serializeStringEscape(f"\\x{code:02x}", char, options)
                ret += esc
            elif 0xd800 <= code <= 0xdfff:
                ret += '�'
            else:
                esc = options.serializeStringEscape(f"\\u{{{code:x}}}", char, options)
                ret += esc
        else:
            ret += char
    ret += options.serializeStringQuote("'", False, options)
    return ret
    
    
def serializeArray(value, depth,options) -> str:
    if len(value) == 0:
        return '[]'
    str_ = '['
    for i, v in enumerate(value):
        if i > 0:
            str_ += ', '
        str_ += serializeImpl(v, depth,options)
    str_ += ']'
    return str_

def serializeRecord(value, depth, options) -> str:
    custom_value = custom_value_of(value)
    if custom_value is not None:
        return serializeImpl(custom_value, depth - 1, options)
    entries = list(value.items())
    if len(entries) == 0:
        return '()'
    if len(entries) == 1:
        k, v = entries[0]
        if k == '0':
            return f"({serializeImpl(v, depth, options)},)"
        return f"({serialize_prop_name(k)}: {serializeImpl(v, depth, options)})"
    omit_key = len(entries) < 10 and all(str(index) == key for index, (key, _) in enumerate(entries))
    str_ = '('
    for idx, (key, val) in enumerate(entries):
        if len(str_) > 1:
            str_ += ', '
        if omit_key:
            str_ += serializeImpl(val, depth, options)
        else:
            str_ += f"{serialize_prop_name(key)}: {serializeImpl(val, depth, options)}"
    str_ += ')'
    return str_

def displayFunction(value):
    try:
        name = getVmFunctionInfo(value)
        if name:
            return f"<function {name}>"
        return "<function>"
    except Exception:
        return "<function>"
class DEFAULT_OPTIONS:
    serializeNil=serializeNil
    serializeBoolean=serializeBoolean
    serializeNumber=serializeNumber
    serializeString=serializeStringImpl
    serializeStringQuote= lambda value: value
    serializeStringEscape=lambda value: value
    serializeStringContent=lambda value: value
    serializeArray = serializeArray
    serializeRecord = serializeRecord
    serializePropName = str
    serializeFunction= serializeNil
    serializeModule = serializeNil
    serializeExtern = serializeNil
# DEFAULT_OPTIONS= DefaultOptions()


def mergeOptions(base,options):
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
        elif char == '\0':
            ret += "\\0"
        elif char == '\n':
            ret += "\\n"
        elif char == '\r':
            ret += "\\r"
        elif char == '\t':
            ret += "\\t"
        elif char == '\b':
            ret += "\\b"
        elif char == '\f':
            ret += "\\f"
        elif char == '\v':
            ret += "\\v"
        elif char == '\\':
            ret += "\\\\"
        elif char == '$':
            ret += "\\$"
        elif re.match(r'\p{C}', char):
            code = ord(char)
            if code <= 0x7f:
                ret += f"\\x{code:02x}"
            elif 0xd800 <= code <= 0xdfff:
                ret += '�'
            else:
                ret += f"\\u{{{code:x}}}"
        else:
            ret += char
    ret += "'"
    return ret

def serialize_prop_name(value: str) -> str:
    if REG_ORDINAL_FULL.fullmatch(value):
        return value
    if REG_IDENTIFIER_FULL.fullmatch(value):
        return value
    return serialize_string(value)






def custom_value_of(value):
    this_value_of = getattr(value, 'valueOf', None)
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
        return options.serializeString(value,  options)
    
    
    if isVmFunction(value):
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
    return options.serializeNil(options)

def serialize(value, options=Uninitialized) -> str:
    return serializeImpl(value, 0,  getSerializeOptions(options))


def displayWrapper(value,useBraces,fallback):
    try:
        return value.toString(useBraces)
    except Exception:
        return fallback


class DISPLAY_OPTIONS:
    serializeNil=serializeNil
    serializeBoolean=serializeBoolean
    serializeNumber=serializeNumber
    serializeString=serializeStringImpl
    serializeStringQuote= lambda value,open,options: value
    serializeStringEscape=lambda value,open,options: value
    serializeStringContent=lambda value,options: value
    serializeArray = serializeArray
    serializeRecord = serializeRecord
    serializePropName = str
    serializeFunction= displayFunction
    serializeModule = lambda value: displayWrapper(value,True,'<module>')
    serializeExtern = lambda value: displayWrapper(value,True,'<extern>')


def display(value, options=Uninitialized):
    opt = mergeOptions(DISPLAY_OPTIONS, options)
    return serializeImpl(value, 0, opt)