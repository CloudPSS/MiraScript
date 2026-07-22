from __future__ import annotations
import math
import re
from typing_extensions import TYPE_CHECKING, override, Final


from .constants import Uninitialized
from .types import is_vm_function, is_vm_extern, is_vm_array, is_vm_module, is_vm_record

if TYPE_CHECKING:
    from .._vm.types import VmFunction, VmModule, VmAny, VmArray, VmExtern, VmRecord


REG_IDENTIFIER: Final = re.compile(r"(?:_+|@+|\$+|[A-Za-z])[A-Za-z0-9_]*", re.UNICODE)
REG_ORDINAL: Final = re.compile(
    r"(?:214748364[0-7]|21474836[0-3]\d|2147483[0-5]\d{2}|214748[0-2]\d{3}|21474[0-7]\d{4}|2147[0-3]\d{5}|214[0-6]\d{6}|21[0-3]\d{7}|20\d{8}|1\d{9}|[1-9]\d{0,8}|0)",
    re.UNICODE,
)


STRING_QUOTE: Final = "'"
STRING_REG: Final = re.compile(r"[\0-\x1f'$\\\u2028\u2029]", re.UNICODE)


def serialize(value: VmAny) -> str:
    return Serializer().serialize(value)


def display(value: VmAny) -> str:
    return DisplaySerializer().serialize(value)


class Serializer:
    """序列化器基类"""

    max_depth: int = 128

    def serialize(self, value: VmAny, depth: int = 0) -> str:
        if value is None or value is Uninitialized:
            return self.serialize_nil()
        if isinstance(value, bool):
            return self.serialize_boolean(value)
        if isinstance(value, (int, float)):
            return self.serialize_number(value)
        if isinstance(value, str):
            return self.serialize_string(value)

        if is_vm_function(value):
            return self.serialize_function(value)
        if is_vm_module(value):
            return self.serialize_module(value, depth + 1)
        if is_vm_extern(value):
            return self.serialize_extern(value, depth + 1)

        if is_vm_array(value):
            return self.serialize_array(value, depth + 1)
        if is_vm_record(value):
            return self.serialize_record(value, depth + 1)

        # 不支持序列化的值
        return self.serialize_nil()

    def serialize_nil(self) -> str:
        return "nil"

    def serialize_boolean(self, value: bool) -> str:
        return "true" if value else "false"

    def serialize_number(self, value: float | int) -> str:
        if math.isnan(value):
            return "nan"
        if not math.isfinite(value):
            return "-inf" if value < 0 else "inf"
        if value == 0:
            if math.copysign(1, value) < 0:
                return "-0"
            return "0"
        return str(value)

    def _serialize_string_escaped(self, value: str) -> str:
        return self.serialize_string_escape("\\" + value)

    def serialize_string(self, value: str) -> str:
        oq = self.serialize_string_quote(STRING_QUOTE, True)
        cq = self.serialize_string_quote(STRING_QUOTE, False)
        if len(value) == 0:
            return oq + cq

        if not STRING_REG.search(value):
            # 不包含特殊字符
            c = self.serialize_string_content(value)
            return oq + c + cq

        ret = oq
        for char in value:
            code = ord(char)
            if char == "'":
                ret += self._serialize_string_escaped("'")
            elif char == "\0":
                ret += self._serialize_string_escaped("0")
            elif char == "\n":
                ret += self._serialize_string_escaped("n")
            elif char == "\r":
                ret += self._serialize_string_escaped("r")
            elif char == "\t":
                ret += self._serialize_string_escaped("t")
            elif char == "\b":
                ret += self._serialize_string_escaped("b")
            elif char == "\f":
                ret += self._serialize_string_escaped("f")
            elif char == "\v":
                ret += self._serialize_string_escaped("v")
            elif char == "\\":
                ret += self._serialize_string_escaped("\\")
            elif char == "$":
                ret += self._serialize_string_escaped("$")
            elif 0xD800 <= code <= 0xDFFF:
                ret += "�"
            elif code == 0x2028 or code == 0x2029:
                ret += self._serialize_string_escaped(f"u{{{code:x}}}")
            elif code <= 0x1F:
                ret += self._serialize_string_escaped(f"x{code:02x}")
            else:
                ret += self.serialize_string_content(char)
        ret += cq
        return ret

    def serialize_string_quote(self, value: str, open: bool) -> str:
        return value

    def serialize_string_escape(self, value: str) -> str:
        return value

    def serialize_string_content(self, value: str) -> str:
        return value

    def serialize_prop_name(self, key: str | int) -> str:
        return str(key)

    def serialize_array(self, value: VmArray, depth: int) -> str:
        if depth > self.max_depth or len(value) == 0:
            return "[]"
        result = "["
        for i, v in enumerate(value):
            if i > 0:
                result += ", "
            result += self.serialize(v, depth)
        result += "]"
        return result

    def _serialize_record_key(self, key: str) -> str:
        if REG_ORDINAL.fullmatch(key):
            return self.serialize_prop_name(int(key))
        if REG_IDENTIFIER.fullmatch(key):
            return self.serialize_prop_name(key)
        return self.serialize_string(key)

    def serialize_record(self, value: VmRecord, depth: int) -> str:
        if depth > self.max_depth or len(value) == 0:
            return "()"
        entries = list(value.items())
        if len(entries) == 1:
            k, v = entries[0]
            if k == "0":
                return f"({self.serialize(v, depth)},)"
            return f"({self._serialize_record_key(k)}: {self.serialize(v, depth)})"
        omit_key = all(str(index) == key for index, (key, _) in enumerate(entries))
        result = "("
        for key, val in entries:
            if len(result) > 1:
                result += ", "
            if omit_key:
                result += self.serialize(val, depth)
            else:
                result += (
                    f"{self._serialize_record_key(key)}: {self.serialize(val, depth)}"
                )
        result += ")"
        return result

    def serialize_function(self, value: VmFunction) -> str:
        return "nil"

    def serialize_module(self, value: VmModule, depth: int) -> str:
        return "nil"

    def serialize_extern(self, value: VmExtern, depth: int) -> str:
        return "nil"


class DisplaySerializer(Serializer):
    """用于显示的序列化选项。"""

    max_depth = 3

    @override
    def serialize_function(self, value: VmFunction) -> str:
        try:
            name = value.__name__
            if name:
                return f"<function {name}>"
            return "<function>"
        except Exception:
            return "<function>"

    @override
    def serialize_module(self, value: VmModule, depth: int) -> str:
        try:
            return str(value)
        except Exception:
            return "<module>"

    @override
    def serialize_extern(self, value: VmExtern, depth: int) -> str:
        try:
            return str(value)
        except Exception:
            return "<extern>"
