import math

from ...._helpers.constants import Uninitialized
from ...._helpers.convert import (
    to_format as _to_format,
    to_number as _to_number,
    to_string as _to_string,
)
from .._helpers import _expect_string, _required


def to_string(value=Uninitialized, fallback=Uninitialized):
    _required("value", value, "")
    return _to_string(value, fallback)


def to_number(value=Uninitialized, fallback=Uninitialized):
    _required("value", value, math.nan)
    x = _to_number(value, fallback)
    return x


def format(value=Uninitialized, fmt=Uninitialized):
    _required("value", value, "")

    return _to_format(value, _expect_string("fmt", fmt))
