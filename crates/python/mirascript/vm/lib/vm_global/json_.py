import json
import math

from mirascript.vm.lib._helpers import _expect_string, _required, _rethrow_error
from mirascript.vm.types.checker import is_vm_module
from mirascript.vm.types.const import Uninitialized
from mirascript.helpers.convert.to_string import innerToString_
from mirascript.vm.types.extern import is_vm_extern


class NanToNullEncoder(json.JSONEncoder):
    def encode(self, o):
        if o is None:
            return "null"
        if isinstance(o, (float)):
            if math.isnan(o) or math.isinf(o):
                return "null"
        if isinstance(o, str):
            return super().encode(o)
        #
        if isinstance(o, (list)):
            r = "[" + ",".join([self.encode(item) for item in o]) + "]"
            return r
        if isinstance(o, (dict)):
            r = "{"
            first = True
            for key in o:
                if not first:
                    r += ","
                first = False
                r += self.encode(str(key))
                r += ":"
                r += self.encode(o[key])
            r += "}"
            return r

        r = innerToString_(o, True)
        return r


def to_json(value=Uninitialized):
    _required("value", value, None)

    if is_vm_module(value) or is_vm_extern(value):
        try:
            return json.dumps(value.value, cls=NanToNullEncoder, ensure_ascii=False)
        except Exception as e:
            _rethrow_error("Failed to convert extern to JSON", e, "{}")

    if callable(value):
        return None
    return json.dumps(value, cls=NanToNullEncoder, ensure_ascii=False)


def from_json(value=Uninitialized, fallback=None):
    _required("value", value, None)
    j = _expect_string("value", value)
    try:

        def parse_constant(x):
            if x == "NaN" or x == "Infinity" or x == "-Infinity":
                raise ValueError(f"{x} is not valid JSON")
            return x

        r = json.loads(j, parse_constant=parse_constant)
        return r
    except Exception as e:
        if fallback is not None:
            return fallback
        _rethrow_error("Invalid JSON", e, None)
