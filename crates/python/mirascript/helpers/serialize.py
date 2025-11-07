import math
from .constants import REG_IDENTIFIER, REG_ORDINAL
from ..vm.types import is_vm_array, is_vm_record

MAX_DEPTH = 100

REG_IDENTIFIER_FULL = REG_IDENTIFIER
REG_ORDINAL_FULL = REG_ORDINAL

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

def serialize_boolean(value: bool) -> str:
    return 'true' if value else 'false'

def serialize_number(value: float) -> str:
    if math.isnan(value):
        return 'nan'
    if not math.isfinite(value):
        return '-inf' if value < 0 else 'inf'
    return str(value)

def serialize_array(value, depth: int) -> str:
    if len(value) == 0:
        return '[]'
    str_ = '['
    for i, v in enumerate(value):
        if i > 0:
            str_ += ', '
        str_ += serialize_impl(v, depth)
    str_ += ']'
    return str_

def custom_value_of(value):
    this_value_of = getattr(value, 'valueOf', None)
    if not callable(this_value_of):
        return None
    custom_value = this_value_of()
    if custom_value is value:
        return None
    return custom_value

def serialize_record(value, depth: int) -> str:
    custom_value = custom_value_of(value)
    if custom_value is not None:
        return serialize_impl(custom_value, depth - 1)
    entries = list(value.items())
    if len(entries) == 0:
        return '()'
    if len(entries) == 1:
        k, v = entries[0]
        if k == '0':
            return f"({serialize_impl(v, depth)},)"
        return f"({serialize_prop_name(k)}: {serialize_impl(v, depth)})"
    omit_key = len(entries) < 10 and all(str(index) == key for index, (key, _) in enumerate(entries))
    str_ = '('
    for idx, (key, val) in enumerate(entries):
        if len(str_) > 1:
            str_ += ', '
        if omit_key:
            str_ += serialize_impl(val, depth)
        else:
            str_ += f"{serialize_prop_name(key)}: {serialize_impl(val, depth)}"
    str_ += ')'
    return str_

def serialize_impl(value, depth: int) -> str:
    if value is None or depth > MAX_DEPTH:
        return 'nil'
    if isinstance(value, bool):
        return serialize_boolean(value)
    if isinstance(value, (int, float)):
        return serialize_number(value)
    if isinstance(value, str):
        return serialize_string(value)
    if is_vm_array(value):
        return serialize_array(value, depth + 1)
    if is_vm_record(value):
        return serialize_record(value, depth + 1)
    # 不支持序列化的值
    return 'nil'

def serialize(value):
    return serialize_impl(value, 0)