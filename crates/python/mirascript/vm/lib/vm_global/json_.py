import json
import math

from mirascript.vm.lib._helpers import _expect_string, _required, _rethrow_error
from mirascript.helpers.types import is_vm_module, is_vm_extern
from mirascript.helpers.constants import Uninitialized
from mirascript.helpers.convert.to_string import toString


class _NanToNullEncoder(json.JSONEncoder):
    def encode(self, o):
        if callable(o):
            return "null"
        if isinstance(o, (float, int, bool)):
            if math.isnan(o) or math.isinf(o):
                return "null"
            return toString(o)

        return super().encode(o)


def to_json(value=Uninitialized):
    _required("value", value, None)

    if is_vm_module(value) or is_vm_extern(value):
        try:
            return json.dumps(value.value, cls=_NanToNullEncoder, ensure_ascii=False)
        except Exception as e:
            _rethrow_error("Failed to convert extern to JSON", e, "{}")

    if callable(value):
        return None
    return json.dumps(value, cls=_NanToNullEncoder, ensure_ascii=False)


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
