from .._helpers import _required, _expect_array
from ...types.types import VmAny
from ...operations import ToString
from ...types.types import Uninitialized

__all__ = [
    "chars",
    "starts_with",
    "ends_with",
    "contains",
    "trim_start",
    "trim_end",
    "trim",
    "replace",
    "split",
    "join",
    "to_uppercase",
    "to_lowercase",
]

_WHITESPACE_CHARS = "\r\n\u2028\u2029\t\v\f\ufeff\x20\xa0\u1680\u2000\u2001\u2002\u2003\u2004\u2005\u2006\u2007\u2008\u2009\u200a\u202f\u205f\u3000"


def chars(string=Uninitialized):
    _required("str", string, None)
    s = ToString(string)
    return list(s)


def starts_with(string=Uninitialized, search=Uninitialized):
    _required("str", string, None)
    _required("search", search, None)
    s = ToString(string)
    p = ToString(search)
    return s.startswith(p)


def ends_with(string=Uninitialized, search=Uninitialized):
    _required("str", string, None)
    _required("search", search, None)
    s = ToString(string)
    suf = ToString(search)
    return s.endswith(suf)


def contains(string=Uninitialized, search=Uninitialized):
    _required("str", string, None)
    _required("search", search, None)
    s = ToString(string)
    sub = ToString(search)
    return sub in s


def trim_start(string=Uninitialized):
    _required("str", string, None)

    s = ToString(string)
    return s.lstrip(_WHITESPACE_CHARS)


def trim_end(string=Uninitialized):
    _required("str", string, None)
    s = ToString(string)
    return s.rstrip(_WHITESPACE_CHARS)


def trim(string=Uninitialized):
    _required("str", string, None)
    s = ToString(string)
    return s.strip(_WHITESPACE_CHARS)


def replace(
    string: VmAny = Uninitialized,
    search: VmAny = Uninitialized,
    replacement: VmAny = "",
):
    string = _required("str", string, None)
    search = _required("search", search, None)
    replacement = _required("replacement", replacement, None)
    s = ToString(string)
    old = ToString(search)
    new = ToString(replacement)
    return s.replace(old, new)


def split(string: VmAny = Uninitialized, separator: VmAny = ""):
    string = _required("str", string, None)
    s = ToString(string)
    sep = ToString(separator)

    if sep == "":
        return list(s)

    return s.split(sep)


def join(string_array: VmAny = Uninitialized, separator: VmAny = ""):
    string_array = _expect_array("arr", string_array, None)
    str_list = [ToString(item) for item in string_array]
    sep = ToString(separator)

    return sep.join(str_list)


def to_uppercase(string=Uninitialized):
    _required("str", string, None)
    s = ToString(string)
    return s.upper()


def to_lowercase(string=Uninitialized):
    _required("str", string, None)
    s = ToString(string)
    return s.lower()
