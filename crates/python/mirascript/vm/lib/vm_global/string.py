from .._helpers import _required, _expect_array
from ...operations import ToString_
from mirascript.vm.types.const import Uninitialized


def chars(string=Uninitialized):
    _required("str", string, None)
    s = ToString_(string)
    return [s[i] for i in range(len(s))]


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
    return s.lstrip("\t\x0b\x0c \xa0 \n\r\ufeff\u2028\u2029")


def trim_end(string=Uninitialized):
    _required("str", string, None)
    s = ToString_(string)
    return s.rstrip("\t\x0b\x0c \xa0 \n\r\ufeff\u2028\u2029")


def trim(string=Uninitialized):
    _required("str", string, None)
    s = ToString_(string)
    return s.strip("\t\x0b\x0c \xa0 \n\r\ufeff\u2028\u2029")


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
        return [s[i] for i in range(len(s))]

    return s.split(sep)


def join(string_array=Uninitialized, separator=""):
    _expect_array("str_array", string_array, None)
    str_list = []
    for item in string_array:
        str_list.append(ToString_(item))
    sep = ToString_(separator)

    return sep.join(str_list)
