import json
import math

from ...._helpers.types import is_vm_module
from ...._helpers.constants import Uninitialized
from ...._helpers.checker import is_number
from ...._helpers.convert.to_string import number_to_string
from ...types import VmAny, VmValue
from .._helpers import _expect_string, _required, _rethrow_error

__all__ = [
    "to_json",
    "from_json",
]


def _purify_json(value: VmValue):
    if value is None or value is Uninitialized:
        return None
    if isinstance(value, (bool, str)):
        return value
    if isinstance(value, (int, float)):
        return None if math.isnan(value) or math.isinf(value) else value
    if callable(value):
        return None
    if isinstance(value, list):
        return [_purify_json(v) for v in value]
    if isinstance(value, dict):
        return {str(k): _purify_json(v) for k, v in value.items()}
    if is_vm_module(value):
        return {k: _purify_json(v) for k, v in value.value.items() if not callable(v)}
    return None


def to_json(value: VmAny = Uninitialized):
    value = _required("value", value, None)

    if callable(value):
        return None

    if is_number(value):
        # 顶层不加 .0，深层的就不管了
        return (
            "null"
            if math.isnan(value) or math.isinf(value)
            else number_to_string(value)
        )

    return json.dumps(_purify_json(value), ensure_ascii=False, separators=(",", ":"))


def from_json(value=Uninitialized, fallback=None):
    value = _expect_string("value", value)
    try:

        def parse_constant(x):
            raise ValueError(f"{x} is not valid JSON")

        r = json.loads(value, parse_constant=parse_constant, parse_int=float)
        return r
    except Exception as e:
        if fallback is not None:
            return fallback
        _rethrow_error("Invalid JSON", e, None)
