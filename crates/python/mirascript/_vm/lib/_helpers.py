import math
from typing_extensions import NoReturn, Sequence

from ..._helpers.types import is_vm_array, is_vm_const, is_vm_primitive, is_vm_record
from ..._helpers.convert import to_number, to_string
from ..._helpers.serialize import display
from ..._helpers.checker import is_safe_integer
from ..operations import Type, Cp
from ..error import VmError
from ..types.types import Uninitialized, VmAny, VmValue


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
    actual = Type(value)
    if isinstance(name, str):
        return _throw_error(
            f"Expected {expected} for parameter '{name}', got {actual}", recovered
        )
    pos = "first" if name <= 0 else "second" if name <= 1 else f"{name+1}th"
    return _throw_error(
        f"Expected {expected} at the {pos} position, got {actual}", recovered
    )


def _rethrow_error(prefix: str, error, recovered) -> NoReturn:
    recovered_value = recovered() if callable(recovered) else recovered
    raise VmError.from_(prefix, error, recovered_value)


def _required(name: "str | int", value: VmAny, recovered) -> VmValue:
    if value is Uninitialized:
        if isinstance(name, str):
            return _throw_error(f"Missing required parameter '{name}'", recovered)
        pos = "first" if name <= 0 else "second" if name <= 1 else f"{name+1}th"
        return _throw_error(
            f"Missing required parameter at the {pos} position", recovered
        )
    return value


def _expect_number(name: "str | int", value: VmAny) -> float:
    value = _required(name, value, math.nan)
    value = to_number(value)
    if value is None:
        _throw_unexpected_type_error(name, "number", value, math.nan)
    return value


def _expect_string(name: str, value: VmAny) -> str:
    value = _required(name, value, "")
    value = to_string(value)
    if value is None:
        _throw_unexpected_type_error(name, "string", value, "")
    return value


def _expect_integer(name: str, value: VmAny) -> float:
    value = _required(name, value, 0)
    value = to_number(value, None)
    if value is None:
        _throw_unexpected_type_error(name, "integer", value, 0)
    from .vm_global.math.round import trunc

    i = trunc(value)
    if not is_safe_integer(i):
        _throw_unexpected_type_error(name, "integer", value, 0)
    return i


def _expect_number_range(name: str, value: VmAny, min_=None, max_=None) -> float:
    value = _expect_number(name, value)
    if not math.isfinite(value):
        _throw_error(
            f"{_describe_param(name)} is less than minimum value {min_}: {display(value)}",
            math.nan,
        )
    if min_ is not None:
        if value < min_:
            _throw_error(
                f"{_describe_param(name)} is less than minimum value {min_}: {display(value)}",
                min_,
            )
    if max_ is not None:
        if value > max_:
            _throw_error(
                f"{_describe_param(name)} is greater than maximum value {max_}: {display(value)}",
                max_,
            )
    return value


def _expect_integer_range(
    name: str, value: VmAny, min_: "int | float", max_: "int | float"
) -> float:
    value = _expect_integer(name, value)
    if value < min_:
        _throw_error(
            f"{_describe_param(name)} is less than minimum value {min_}: {display(value)}",
            min_,
        )
    if value > max_:
        _throw_error(
            f"{_describe_param(name)} is greater than maximum value {max_}: {display(value)}",
            max_,
        )
    return value


def _expect_array(name, value, recovered):
    _required(name, value, recovered)
    if not is_vm_array(value):
        _throw_unexpected_type_error(name, "array", value, recovered)
    return value


def _expect_record(name, value, recovered):
    _required(name, value, recovered)
    if not is_vm_record(value):
        _throw_unexpected_type_error(name, "record", value, recovered)
    return value


def _expect_array_or_record(name, value, recovered):
    _required(name, value, recovered)
    if not is_vm_array(value) and not is_vm_record(value):
        _throw_unexpected_type_error(name, "array | record", value, recovered)
    return value


def _expect_compound(name, value, recovered):
    _required(name, value, recovered)
    if is_vm_primitive(value) or callable(value):
        _throw_unexpected_type_error(
            name, "array | record | module | extern", value, recovered
        )
    return value


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


def _get_numbers(args: "Sequence[VmValue] | None") -> "list[float]":
    if not args:
        return []
    useFirst = False
    if len(args) == 1 and is_vm_array(args[0]):
        args = args[0]
        useFirst = True
    numbers = []
    for i in range(len(args)):
        arg = args[i]
        numbers.append(_expect_number(name="values" if useFirst else i, value=arg))
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
