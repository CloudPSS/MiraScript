from __future__ import annotations

from mirascript import (
    is_vm_script,
    # is_vm_context,
    # is_vm_function,
    # is_vm_module,
    # is_vm_extern,
    # is_vm_wrapper,
    # is_vm_callable,
    # is_vm_primitive,
    is_vm_array,
    is_vm_record,
    is_vm_const,
    is_vm_immutable,
    is_vm_any,
    is_vm_value,
    get_vm_type,
    Uninitialized,
    vm_function,
    VmModule,
    compile,
)

mod = VmModule("test", {"x": 1})
func = vm_function(lambda x: x)
script = compile("1")[0]


def test_get_vm_type():
    assert get_vm_type(Uninitialized) == "uninitialized"
    assert get_vm_type(None) == "nil"
    assert get_vm_type(True) == "boolean"
    assert get_vm_type(1) == "number"
    assert get_vm_type("abc") == "string"
    assert get_vm_type([]) == "array"
    assert get_vm_type({}) == "record"
    assert get_vm_type(lambda x: x) == "unknown"
    assert get_vm_type(func) == "function"
    assert get_vm_type(mod) == "module"


def test_is_vm_script():
    assert not is_vm_script(None)
    assert not is_vm_script(lambda x: x)
    assert not is_vm_script(func)
    assert is_vm_script(script)


def test_is_vm_array():
    assert not is_vm_array(None)
    assert not is_vm_array(func)
    assert is_vm_array([])
    assert is_vm_array([1, 2, ["x"], mod])  # pyright: ignore[reportArgumentType]
    assert not is_vm_array({})
    assert not is_vm_array(mod)


def test_is_vm_record():
    assert not is_vm_record(None)
    assert not is_vm_record(func)
    assert not is_vm_record([])
    assert is_vm_record({})
    assert is_vm_record(
        {"x": 1, "y": [1, 2], "z": mod}  # pyright: ignore[reportArgumentType]
    )
    assert not is_vm_record(mod)


def test_is_vm_const():
    assert is_vm_const(None)
    assert is_vm_const(True)
    assert is_vm_const(1)
    assert is_vm_const("abc")
    assert is_vm_const([])
    assert is_vm_const([1, 2, ["x"], mod])  # pyright: ignore[reportArgumentType]
    assert not is_vm_const([1, 2, ["x"], mod], True)
    assert is_vm_const({})
    assert is_vm_const(
        {"x": 1, "y": [1, 2], "z": mod}  # pyright: ignore[reportArgumentType]
    )
    assert not is_vm_const({"x": 1, "y": [1, 2], "z": mod}, True)
    assert not is_vm_const(func)
    assert not is_vm_const(mod)


def test_is_vm_immutable():
    assert is_vm_immutable(None)
    assert is_vm_immutable(True)
    assert is_vm_immutable(1)
    assert is_vm_immutable("abc")
    assert is_vm_immutable([])
    assert is_vm_immutable([1, 2, ["x"], mod])  # pyright: ignore[reportArgumentType]
    assert not is_vm_immutable([1, 2, ["x"], mod], True)
    assert is_vm_immutable({})
    assert is_vm_immutable(
        {"x": 1, "y": [1, 2], "z": mod}  # pyright: ignore[reportArgumentType]
    )
    assert not is_vm_immutable({"x": 1, "y": [1, 2], "z": mod}, True)
    assert is_vm_immutable(func)
    assert is_vm_immutable(mod)


def test_is_vm_any():
    assert is_vm_any(Uninitialized, False)
    assert is_vm_any(None, False)
    assert is_vm_any(True, False)
    assert is_vm_any(1, False)
    assert is_vm_any("abc", False)
    assert is_vm_any([], False)
    assert not is_vm_any([[[mod]]], True)
    assert not is_vm_any({"x": {1: 2}}, True)
    assert not is_vm_any({"x": {"y": mod}}, True)
    assert is_vm_any({}, False)
    assert not is_vm_any(lambda x: x, False)
    assert is_vm_any(func, False)
    assert is_vm_any(mod, False)
    assert not is_vm_any(script, False)


def test_is_vm_value():
    assert not is_vm_value(Uninitialized, False)
    assert is_vm_value(None, False)
    assert is_vm_value(True, False)
    assert is_vm_value(1, False)
    assert is_vm_value("abc", False)
    assert is_vm_value([], False)
    assert is_vm_value({}, False)
    assert not is_vm_value(lambda x: x, False)
    assert is_vm_value(func, False)
    assert is_vm_value(mod, False)
    assert not is_vm_value(script, True)
