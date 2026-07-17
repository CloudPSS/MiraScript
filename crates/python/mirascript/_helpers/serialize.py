from dataclasses import dataclass
import math
import re
from typing_extensions import Callable, Optional, TYPE_CHECKING

from .constants import Uninitialized
from .types import is_vm_function, is_vm_extern, is_vm_array, is_vm_module, is_vm_record

if TYPE_CHECKING:
    from .._vm.types import VmFunction, VmModule, VmAny, VmArray, VmExtern, VmRecord

REG_IDENTIFIER = re.compile(r"(?:_+|@+|\$+|[A-Za-z])[A-Za-z0-9_]*", re.UNICODE)
REG_ORDINAL = re.compile(
    r"(?:214748364[0-7]|21474836[0-3]\d|2147483[0-5]\d{2}|214748[0-2]\d{3}|21474[0-7]\d{4}|2147[0-3]\d{5}|214[0-6]\d{6}|21[0-3]\d{7}|20\d{8}|1\d{9}|[1-9]\d{0,8}|0)",
    re.UNICODE,
)


def serialize_nil(options: Optional["SerializeOptions"] = None) -> str:
    return "nil"


def serialize_boolean(value: bool, options: Optional["SerializeOptions"] = None) -> str:
    return "true" if value else "false"


def serialize_number(
    value: "float | int", options: Optional["SerializeOptions"] = None
) -> str:
    if math.isnan(value):
        return "nan"
    if not math.isfinite(value):
        return "-inf" if value < 0 else "inf"
    if value == 0:
        if math.copysign(1, value) < 0:
            return "-0"
        return "0"
    return str(value)


def _serialize_string_escaped(escaped: str, options: "SerializeOptions") -> str:
    return options.serialize_string_escape("\\" + escaped, options)


def _serialize_string_content(content: str, options: "SerializeOptions") -> str:
    return options.serialize_string_content(content, options)


STRING_QUOTE = "'"
STRING_REG = re.compile(r"[\0-\x1f'$\\\u2028\u2029]", re.UNICODE)


def _serialize_string_impl(value: str, options: "SerializeOptions") -> str:
    oq = options.serialize_string_quote(STRING_QUOTE, True, options)
    cq = options.serialize_string_quote(STRING_QUOTE, False, options)
    if len(value) == 0:
        return oq + cq

    if not STRING_REG.search(value):
        # 不包含特殊字符
        c = _serialize_string_content(value, options)
        return oq + c + cq

    ret = oq
    for char in value:
        code = ord(char)
        if char == "'":
            ret += _serialize_string_escaped("'", options)
        elif char == "\0":
            ret += _serialize_string_escaped("0", options)
        elif char == "\n":
            ret += _serialize_string_escaped("n", options)
        elif char == "\r":
            ret += _serialize_string_escaped("r", options)
        elif char == "\t":
            ret += _serialize_string_escaped("t", options)
        elif char == "\b":
            ret += _serialize_string_escaped("b", options)
        elif char == "\f":
            ret += _serialize_string_escaped("f", options)
        elif char == "\v":
            ret += _serialize_string_escaped("v", options)
        elif char == "\\":
            ret += _serialize_string_escaped("\\", options)
        elif char == "$":
            ret += _serialize_string_escaped("$", options)
        elif 0xD800 <= code <= 0xDFFF:
            ret += "�"
        elif code == 0x2028 or code == 0x2029:
            ret += _serialize_string_escaped(f"u{{{code:x}}}", options)
        elif code <= 0x1F:
            ret += _serialize_string_escaped(f"x{code:02x}", options)
        else:
            ret += char
    ret += cq
    return ret


def serialize_string(value: str, options: Optional["SerializeOptions"] = None) -> str:
    if options is None:
        options = SerializeOptions()
    return _serialize_string_impl(value, options)


def serialize_record_key(key: str, options: "SerializeOptions") -> str:
    if REG_ORDINAL.fullmatch(key):
        return options.serialize_prop_name(int(key), options)
    if REG_IDENTIFIER.fullmatch(key):
        return options.serialize_prop_name(key, options)
    return options.serialize_string(key, options)


def _serialize_prop_name(key: str | int, options: Optional["SerializeOptions"]) -> str:
    return str(key) if isinstance(key, int) else key


def serialize_array(value: "VmArray", depth: int, options: "SerializeOptions") -> str:
    if depth > options.max_depth or len(value) == 0:
        return "[]"
    result = "["
    for i, v in enumerate(value):
        if i > 0:
            result += ", "
        result += _serialize_impl(v, depth, options)
    result += "]"
    return result


def serialize_record(value: "VmRecord", depth: int, options: "SerializeOptions") -> str:
    if depth > options.max_depth or len(value) == 0:
        return "()"
    entries = list(value.items())
    if len(entries) == 1:
        k, v = entries[0]
        if k == "0":
            return f"({_serialize_impl(v, depth, options)},)"
        return f"({serialize_record_key(k, options)}: {_serialize_impl(v, depth, options)})"
    omit_key = all(str(index) == key for index, (key, _) in enumerate(entries))
    result = "("
    for key, val in entries:
        if len(result) > 1:
            result += ", "
        if omit_key:
            result += _serialize_impl(val, depth, options)
        else:
            result += f"{serialize_record_key(key, options)}: {_serialize_impl(val, depth, options)}"
    result += ")"
    return result


def _serialize_impl(value: "VmAny", depth: int, options: "SerializeOptions") -> str:
    if value is None or value is Uninitialized:
        return options.serialize_nil(options)
    if isinstance(value, bool):
        return options.serialize_boolean(value, options)
    if isinstance(value, (int, float)):
        return options.serialize_number(value, options)
    if isinstance(value, str):
        return options.serialize_string(value, options)

    if is_vm_function(value):
        return options.serialize_function(value, options)
    if is_vm_module(value):
        return options.serialize_module(value, depth + 1, options)
    if is_vm_extern(value):
        return options.serialize_extern(value, depth + 1, options)

    if is_vm_array(value):
        return options.serialize_array(value, depth + 1, options)
    if is_vm_record(value):
        return options.serialize_record(value, depth + 1, options)

    # 不支持序列化的值
    return options.serialize_nil(options)


def serialize(value: "VmAny", options: Optional["SerializeOptions"] = None) -> str:
    return _serialize_impl(value, 0, SerializeOptions() if options is None else options)


def display_function(
    value: "VmFunction", options: Optional["SerializeOptions"] = None
) -> str:
    try:
        name = value.__name__
        if name:
            return f"<function {name}>"
        return "<function>"
    except Exception:
        return "<function>"


def display_module(
    value: "VmModule",
    depth: Optional[int] = None,
    options: Optional["SerializeOptions"] = None,
) -> str:
    try:
        return str(value)
    except Exception:
        return "<module>"


def display_extern(
    value: "VmExtern",
    depth: Optional[int] = None,
    options: Optional["SerializeOptions"] = None,
) -> str:
    try:
        return str(value)
    except Exception:
        return "<extern>"


def display(value: "VmAny", options: Optional["DisplayOptions"] = None) -> str:
    return _serialize_impl(value, 0, DisplayOptions() if options is None else options)


@dataclass
class SerializeOptions:
    max_depth: int = 128
    serialize_nil: "Callable[[SerializeOptions], str]" = serialize_nil
    serialize_boolean: "Callable[[bool, SerializeOptions], str]" = serialize_boolean
    serialize_number: "Callable[[float | int, SerializeOptions], str]" = (
        serialize_number
    )
    serialize_string: "Callable[[str, SerializeOptions], str]" = _serialize_string_impl
    serialize_string_quote: "Callable[[str, bool, SerializeOptions], str]" = (
        lambda value, open, options: value
    )
    serialize_string_escape: "Callable[[str, SerializeOptions], str]" = (
        lambda value, options: value
    )
    serialize_string_content: "Callable[[str, SerializeOptions], str]" = (
        lambda value, options: value
    )
    serialize_array: "Callable[[VmArray, int, SerializeOptions], str]" = serialize_array
    serialize_record: "Callable[[VmRecord, int, SerializeOptions], str]" = (
        serialize_record
    )
    serialize_prop_name: "Callable[[str | int, SerializeOptions], str]" = (
        _serialize_prop_name
    )
    serialize_function: "Callable[[VmFunction, SerializeOptions], str]" = (
        lambda value, options: serialize_nil(options)
    )
    serialize_module: "Callable[[VmModule, int, SerializeOptions], str]" = (
        lambda value, depth, options: serialize_nil(options)
    )
    serialize_extern: "Callable[[VmExtern, int, SerializeOptions], str]" = (
        lambda value, depth, options: serialize_nil(options)
    )


@dataclass
class DisplayOptions(SerializeOptions):
    max_depth: int = 3
    serialize_function: "Callable[[VmFunction, SerializeOptions], str]" = (
        display_function
    )
    serialize_module: "Callable[[VmModule, int, SerializeOptions], str]" = (
        display_module
    )
    serialize_extern: "Callable[[VmExtern, int, SerializeOptions], str]" = (
        display_extern
    )
