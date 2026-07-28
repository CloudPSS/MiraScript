from __future__ import annotations

import math

import pytest

from mirascript import (
    is_vm_script,
    # is_vm_context,
    # is_vm_function,
    # is_vm_module,
    # is_vm_extern,
    # is_vm_wrapper,
    # is_vm_callable,
    # is_vm_primitive,
    # is_vm_array,
    # is_vm_record,
    # is_vm_const,
    # is_vm_immutable,
    # is_vm_any,
    # is_vm_value,
    get_vm_type,
    Uninitialized,
    vm_function,
    VmModule,
    compile,
)


def test_get_vm_type():
    assert get_vm_type(Uninitialized) == "uninitialized"
    assert get_vm_type(None) == "nil"
    assert get_vm_type(True) == "boolean"
    assert get_vm_type(1) == "number"
    assert get_vm_type("abc") == "string"
    assert get_vm_type([]) == "array"
    assert get_vm_type({}) == "record"
    assert get_vm_type(lambda x: x) == "unknown"
    assert get_vm_type(vm_function(lambda x: x)) == "function"
    assert get_vm_type(VmModule("mod", {})) == "module"


def test_is_vm_script():
    assert not is_vm_script(None)
    assert not is_vm_script(lambda x: x)
    assert is_vm_script(compile("1")[0])
