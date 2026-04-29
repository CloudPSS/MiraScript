from .._helpers import _required, _expect_array
from ...operations import ToString_
from mirascript.vm.types.const import Uninitialized

_WHITESPACE_CHARS = "\r\n\u2028\u2029\t\v\f\ufeff\x20\xa0\u1680\u2000\u2001\u2002\u2003\u2004\u2005\u2006\u2007\u2008\u2009\u200a\u202f\u205f\u3000"


def chars(string=Uninitialized):
    _required("str", string, None)
    s = ToString_(string)
    return list(s)


def starts_with(string=Uninitialized, search=Uninitialized):
    _required("str", string, None)
    _required("search", search, None)
    s = ToString_(string)
    p = ToString_(search)
    return s.startswith(p)


def ends_with(string=Uninitialized, search=Uninitialized):
    _required("str", string, None)
    _required("search", search, None)
    s = ToString_(string)
    suf = ToString_(search)
    return s.endswith(suf)


def contains(string=Uninitialized, search=Uninitialized):
    _required("str", string, None)
    _required("search", search, None)
    s = ToString_(string)
    sub = ToString_(search)
    return sub in s


def trim_start(string=Uninitialized):
    _required("str", string, None)

    s = ToString_(string)
    return s.lstrip(_WHITESPACE_CHARS)


def trim_end(string=Uninitialized):
    _required("str", string, None)
    s = ToString_(string)
    return s.rstrip(_WHITESPACE_CHARS)


def trim(string=Uninitialized):
    _required("str", string, None)
    s = ToString_(string)
    return s.strip(_WHITESPACE_CHARS)


def replace(string=Uninitialized, search=Uninitialized, replacement=""):
    _required("str", string, None)
    _required("search", search, None)
    _required("replacement", replacement, None)
    s = ToString_(string)
    old = ToString_(search)
    new = ToString_(replacement)
    return s.replace(old, new)


def split(string=Uninitialized, separator=""):
    _required("str", string, None)
    s = ToString_(string)
    sep = ToString_(separator)

    if sep == "":
        return list(s)

    return s.split(sep)


def join(string_array=Uninitialized, separator=""):
    _expect_array("arr", string_array, None)
    str_list = [ToString_(item) for item in string_array]
    sep = ToString_(separator)

    return sep.join(str_list)


def to_uppercase(string=Uninitialized):
    _required("str", string, None)
    s = ToString_(string)
    return s.upper()


def to_lowercase(string=Uninitialized):
    _required("str", string, None)
    s = ToString_(string)
    return s.lower()
