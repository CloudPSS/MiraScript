from __future__ import annotations

import pytest

from mirascript import Uninitialized, VmError, VmModule, operations

mod = VmModule("test", {"x": 1})


def test_op_in():
    assert operations.In(1, [1, 2, 3])
    assert operations.In(-0.0, [0, 2, 3])
    assert operations.In(None, [None])
    assert not operations.In("a", "abc")
    assert operations.In("a", {"a": 1})
    assert operations.In("x", mod)
    assert not operations.In(None, None)
    with pytest.raises(VmError):
        operations.In("y", Uninitialized)


def test_op_length():
    assert operations.Length([1, 2, 3]) == 3
    assert operations.Length({"a": 1, "b": 2}) == 2
    with pytest.raises(TypeError):
        operations.Length("abc")
    with pytest.raises(TypeError):
        operations.Length(1)
    with pytest.raises(TypeError):
        operations.Length(None)
    with pytest.raises(TypeError):
        operations.Length(mod)


def test_op_omit_pick():

    assert operations.Omit({"a": 1, "b": 2}, ["a", "x"]) == {"b": 2}
    assert operations.Omit([1, 2, 3], [0]) == {}
    assert operations.Omit(1, [0]) == {}
    assert operations.Omit(mod, [0]) == {}
    assert operations.Pick({"a": 1, "b": 2}, ["a", "x"]) == {"a": 1}
    assert operations.Pick([1, 2, 3], [0]) == {}
    assert operations.Pick(1, [0]) == {}
    assert operations.Pick(mod, ["x"]) == {}


def test_op_has():

    assert operations.Has({"a": 1}, "a")
    assert operations.Has([1, 2], 1)
    assert not operations.Has([1, 2], 1.5)
    assert not operations.Has(1, "a")
    assert not operations.Has(None, "a")
    assert operations.Has(mod, "x")
    assert not operations.Has(mod, "y")


def test_op_get():

    assert operations.Get([10], 0) == 10
    assert operations.Get([10], 99) is None
    assert operations.Get([10], -1) == 10
    assert operations.Get({"a": 1}, "a") == 1
    assert operations.Get("abc", 0) is None


def test_op_set():

    with pytest.raises(VmError):
        operations.Set({}, "a", 1)

    with pytest.raises(VmError):
        operations.Set([1], 0, 1)

    with pytest.raises(VmError):
        operations.Set("x", 0, 1)


def test_op_iterable():
    assert list(operations.Iterable({"a": 1, "b": 2})) == ["a", "b"]
    assert list(operations.Iterable([1, 2, 3])) == [1, 2, 3]
    with pytest.raises(VmError):
        operations.Iterable(None)
    with pytest.raises(VmError):
        operations.Iterable("x")
    with pytest.raises(VmError):
        operations.Iterable(1)


def test_op_record_spread():
    assert operations.RecordSpread({"a": 1, "b": 2}) == {"a": 1, "b": 2}
    assert operations.RecordSpread([1, 2]) == {"0": 1, "1": 2}
    assert operations.RecordSpread(None) == {}
    with pytest.raises(VmError):
        operations.RecordSpread(1)


def test_op_array_spread():
    assert list(operations.ArraySpread([1, 2, 3])) == [1, 2, 3]
    assert list(operations.ArraySpread(None)) == []
    with pytest.raises(VmError):
        operations.ArraySpread(1)
    with pytest.raises(VmError):
        operations.ArraySpread({"a": 1})
