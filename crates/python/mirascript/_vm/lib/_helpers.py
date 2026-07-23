from __future__ import annotations
import math
from typing_extensions import NoReturn, Sequence, TYPE_CHECKING

from ..._helpers.types import is_vm_array, is_vm_const, is_vm_primitive, is_vm_record
from ..._helpers.convert import to_number, to_string
from ..._helpers.serialize import display
from ..._helpers.checker import is_safe_integer
from ..operations import Type, Cp
from ..error import VmError
from ..types import Uninitialized

if TYPE_CHECKING:
    from ..types import (
        VmFunction,
        VmModule,
        VmExtern,
        VmValue,
        VmAny,
        VmArray,
        VmRecord,
        VmConst,
    )


def _describe_param(name):
    if name is None:
        return "Value"
    if isinstance(name, str):
        if name == "":
            return "Argument"
        return f"parameter '{name}'"
    pos = "first" if name <= 0 else "second" if name <= 1 else f"{name+1}th"
    return f"parameter at the {pos} position"


def _throw_error(message: str, recovered: VmAny) -> NoReturn:
    recovered_value = recovered() if callable(recovered) else recovered
    raise VmError(message, recovered_value)


def _throw_unexpected_type_error(
    name: str | int, expected: str, value: VmAny, recovered: VmAny
) -> NoReturn:
    actual = Type(value)
    if isinstance(name, str):
        return _throw_error(
            f"Expected {expected} for parameter '{name}', got {actual}", recovered
        )
    pos = "first" if name <= 0 else "second" if name <= 1 else f"{name+1}th"
    return _throw_error(
        f"Expected {expected} at the {pos} position, got {actual}", recovered
    )


def _rethrow_error(prefix: str, error: Exception, recovered: VmAny) -> NoReturn:
    recovered_value = recovered() if callable(recovered) else recovered
    raise VmError.from_(prefix, error, recovered_value)


def _required(name: str | int, value: VmAny, recovered: VmAny) -> VmValue:
    if value is Uninitialized:
        if isinstance(name, str):
            return _throw_error(f"Missing required parameter '{name}'", recovered)
        pos = "first" if name <= 0 else "second" if name <= 1 else f"{name+1}th"
        return _throw_error(
            f"Missing required parameter at the {pos} position", recovered
        )
    return value


def _expect_number(name: str | int, value: VmAny) -> float:
    if isinstance(value, float):
        return value
    value = _required(name, value, math.nan)
    value = to_number(value)
    if value is None:
        _throw_unexpected_type_error(name, "number", value, math.nan)
    return value


def _expect_string(name: str | int, value: VmAny) -> str:
    if isinstance(value, str):
        return value
    value = _required(name, value, "")
    value = to_string(value)
    if value is None:
        _throw_unexpected_type_error(name, "string", value, "")
    return value


def _expect_integer(name: str | int, value: VmAny) -> float:
    value = _required(name, value, 0)
    value = to_number(value, None)
    if value is None:
        return _throw_unexpected_type_error(name, "integer", value, 0)
    from .vm_global.math.round import trunc

    i = trunc(value)
    if not is_safe_integer(i):
        _throw_unexpected_type_error(name, "integer", value, 0)
    return i


def _expect_number_range(
    name: str | int, value: VmAny, min: float | None = None, max: float | None = None
) -> float:
    value = _expect_number(name, value)
    if not math.isfinite(value):
        return _throw_error(
            f"{_describe_param(name)} is less than minimum value {min}: {display(value)}",
            math.nan,
        )
    if min is not None:
        if value < min:
            return _throw_error(
                f"{_describe_param(name)} is less than minimum value {min}: {display(value)}",
                min,
            )
    if max is not None:
        if value > max:
            return _throw_error(
                f"{_describe_param(name)} is greater than maximum value {max}: {display(value)}",
                max,
            )
    return value


def _expect_integer_range(
    name: str | int, value: VmAny, min: int | float, max: int | float
) -> float:
    value = _expect_integer(name, value)
    if value < min:
        return _throw_error(
            f"{_describe_param(name)} is less than minimum value {min}: {display(value)}",
            min,
        )
    if value > max:
        return _throw_error(
            f"{_describe_param(name)} is greater than maximum value {max}: {display(value)}",
            max,
        )
    return value


def _expect_array(name: str | int, value: VmAny, recovered: VmAny) -> VmArray:
    value = _required(name, value, recovered)
    if not is_vm_array(value):
        return _throw_unexpected_type_error(name, "array", value, recovered)
    return value


def _expect_record(name: str | int, value: VmAny, recovered: VmAny) -> VmRecord:
    value = _required(name, value, recovered)
    if not is_vm_record(value):
        return _throw_unexpected_type_error(name, "record", value, recovered)
    return value


def _expect_array_or_record(
    name: str, value: VmAny, recovered: VmAny
) -> VmArray | VmRecord:
    value = _required(name, value, recovered)
    if not is_vm_array(value) and not is_vm_record(value):
        return _throw_unexpected_type_error(name, "array | record", value, recovered)
    return value


def _expect_compound(
    name: str, value: VmAny, recovered: VmAny
) -> VmArray | VmRecord | VmModule | VmExtern:
    value = _required(name, value, recovered)
    if is_vm_primitive(value) or callable(value):
        return _throw_unexpected_type_error(
            name, "array | record | module | extern", value, recovered
        )
    return value


def _expect_const(name: str | int, value: VmAny, recovered: VmAny) -> VmConst:
    value = _required(name, value, recovered)
    if not is_vm_const(value):
        return _throw_unexpected_type_error(
            name, "nil | number | boolean | string | array | record", value, recovered
        )
    return value


def _expect_callable(name: str | int, value: VmAny, recovered: VmAny) -> VmFunction:
    _required(name, value, recovered)
    if not callable(value):
        return _throw_unexpected_type_error(name, "callable", value, recovered)
    return value


def _get_numbers(args: Sequence[VmValue] | None) -> list[float]:
    if not args:
        return []
    use_first = False
    if len(args) == 1 and is_vm_array(args[0]):
        args = args[0]
        use_first = True
    numbers: list[float] = [0.0] * len(args)
    for i, arg in enumerate(args):
        if isinstance(arg, float):
            numbers[i] = arg
            continue
        arg = _expect_number("values" if use_first else i, arg)
        numbers[i] = arg
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
