from mirascript.helpers.convert.to_boolean import toBoolean
from mirascript.helpers.convert.to_format import toFormat
from mirascript.helpers.convert.to_number import toNumber
from mirascript.helpers.convert.to_string import toString

from .._helpers import _expect_string, _required
import math
from mirascript.vm.types.const import Uninitialized


def to_string(value=Uninitialized, fallback=Uninitialized):
    _required("value", value, "")
    return toString(value, fallback)


def to_number(value=Uninitialized, fallback=Uninitialized):
    _required("value", value, math.nan)
    x = toNumber(value, fallback)
    return x


# def to_boolean(value=Uninitialized, fallback=Uninitialized):
#     _required("value", value, False)
#     return toBoolean(value, fallback)


def format(value=Uninitialized, fmt=Uninitialized):
    _required("value", value, "")

    return toFormat(value, _expect_string("fmt", fmt))
