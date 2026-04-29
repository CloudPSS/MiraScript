import math
from typing import NoReturn

from mirascript.helpers.types import (
    is_vm_array,
    is_vm_const,
    is_vm_primitive,
    is_vm_record,
)

from ...helpers.convert.to_number import toNumber
from ...helpers.convert.to_string import toString
from ...helpers.serialize import display

from ..operations import Type_, is_safe_integer
from ..error import VmError
from ..helpers import Cp
from ..types.types import Uninitialized


def _describe_param(name):
    if name is None:
        return "Value"
    if isinstance(name, str):
        if name == "":
            return "Argument"
        return f"parameter '{name}'"
    pos = "first" if name <= 0 else "second" if name <= 1 else f"{name+1}th"
    return f"parameter at the {pos} position"


def _throw_error(message: str, recovered) -> NoReturn:
    recovered_value = recovered() if callable(recovered) else recovered
    raise VmError(message, recovered_value)


def _throw_unexpected_type_error(name, expected, value, recovered) -> NoReturn:
    actual = Type_(value)
    if isinstance(name, str):
        return _throw_error(
            f"Expected {expected} for parameter '{name}', got {actual}", recovered
        )
    pos = "first" if name <= 0 else "second" if name <= 1 else f"{name+1}th"
    _throw_error(f"Expected {expected} at the {pos} position, got {actual}", recovered)


def _rethrow_error(prefix: str, error, recovered) -> NoReturn:
    recovered_value = recovered() if callable(recovered) else recovered
    raise VmError.from_(prefix, error, recovered_value)


def _required(name, value, recovered):
    if value is Uninitialized:
        if isinstance(name, str):
            _throw_error(f"Missing required parameter '{name}'", recovered)
            return
        pos = "first" if name <= 0 else "second" if name <= 1 else f"{name+1}th"
        _throw_error(f"Missing required parameter at the {pos} position", recovered)


def _expect_number(name, value) -> float:
    _required(name, value, math.nan)
    v = toNumber(value)
    if v is None:
        _throw_unexpected_type_error(name, "number", value, math.nan)
    return v


def _expect_string(name, value) -> str:
    _required(name, value, "")
    v = toString(value)
    if v is None:
        _throw_unexpected_type_error(name, "string", value, "")
    return v


def _expect_integer(name, value) -> float:
    _required(name, value, 0)
    v = toNumber(value, None)
    if v is None:
        _throw_unexpected_type_error(name, "integer", value, 0)
    from mirascript.vm.lib.vm_global.math.round import trunc

    i = trunc(v)
    if not is_safe_integer(i):
        _throw_unexpected_type_error(name, "integer", value, 0)
    return i


def _expect_number_range(name, value, min_=None, max_=None):
    v = _expect_number(name, value)
    if not math.isfinite(v):
        _throw_error(
            f"{_describe_param(name)} is less than minimum value {min_}: {display(value)}",
            math.nan,
        )
    if min_ is not None:
        if v < min_:
            _throw_error(
                f"{_describe_param(name)} is less than minimum value {min_}: {display(value)}",
                min_,
            )
    if max_ is not None:
        if v > max_:
            _throw_error(
                f"{_describe_param(name)} is greater than maximum value {max_}: {display(value)}",
                max_,
            )
    return v


def _expect_integer_range(name, value, min_, max_):
    v = _expect_integer(name, value)
    if v < min_:
        _throw_error(
            f"{_describe_param(name)} is less than minimum value {min_}: {display(value)}",
            min_,
        )
    if v > max_:
        _throw_error(
            f"{_describe_param(name)} is greater than maximum value {max_}: {display(value)}",
            max_,
        )
    return v


def _expect_array(name, value, recovered):
    _required(name, value, recovered)
    if not is_vm_array(value):
        _throw_unexpected_type_error(name, "array", value, recovered)


def _expect_record(name, value, recovered):
    _required(name, value, recovered)
    if not is_vm_record(value):
        _throw_unexpected_type_error(name, "record", value, recovered)


def _expect_array_or_record(name, value, recovered):
    _required(name, value, recovered)
    if not is_vm_array(value) and not is_vm_record(value):
        _throw_unexpected_type_error(name, "array | record", value, recovered)


def _expect_compound(name, value, recovered):
    _required(name, value, recovered)
    if is_vm_primitive(value) or callable(value):
        _throw_unexpected_type_error(
            name, "array | record | module | extern", value, recovered
        )


def _expect_const(name, value, recovered):
    _required(name, value, recovered)
    if not is_vm_const(value):
        _throw_unexpected_type_error(
            name, "nil | number | boolean | string | array | record", value, recovered
        )


def _expect_callable(name, value, recovered):
    _required(name, value, recovered)
    callable_ = callable(value)
    if not callable_:
        _throw_unexpected_type_error(name, "callable", value, recovered)


def _get_numbers(args):
    if not args:
        return []
    useFirst = False
    if len(args) == 1 and is_vm_array(args[0]):
        args = args[0]
        useFirst = True
    numbers = []
    # for arg in args:
    for i in range(len(args)):
        arg = args[i]
        numbers.append(_expect_number(name=None if useFirst else i, value=arg))
    return numbers


def _map_vm(data, mapper):
    if is_vm_primitive(data):
        ret = mapper(data, None, data)
        if ret is Uninitialized:
            return None
        return ret
    if is_vm_array(data):
        result = []

        for i, v in enumerate(data):
            Cp()
            ret = mapper(v if v is not None else None, i, data)
            if ret is Uninitialized:
                continue
            if is_vm_const(ret):
                result.append(ret)
            else:
                result.append(None)
        return result
    else:
        entries = []
        for key, value in data.items():
            Cp()
            ret = mapper(value if value is not None else None, key, data)
            if ret is Uninitialized:
                continue
            if is_vm_const(ret):
                entries.append((key, ret))
            else:
                entries.append((key, None))
        return dict(entries)
