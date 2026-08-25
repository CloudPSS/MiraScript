from __future__ import annotations

import struct
import sys
import types

import pytest

from mirascript._compiler.consts import (
    _read_constant,
    read_constants,
    read_index,
    read_param,
    split_chunk,
)
from mirascript._compiler.diagnostics import (
    Diagnostic,
    SourceMapEntry,
    decode_diagnostics,
)
from mirascript._compiler.opcode import OpCode, get_opcode_name
from mirascript._compiler.script import wrap_vm_script
from mirascript._compiler.core import Config


def test_compile_config():
    config = Config()
    assert isinstance(config, Config)

    with pytest.raises(ValueError):
        Config(input_mode="invalid")


def test_consts_read_constant_and_errors():
    assert _read_constant(bytes([0]), 0) == (None, 1)
    assert _read_constant(bytes([1]), 0) == (True, 1)
    assert _read_constant(bytes([2]), 0) == (False, 1)

    ordinal = bytes([3]) + struct.pack("<i", 7)
    assert _read_constant(ordinal, 0) == (7.0, 5)

    num = bytes([4]) + struct.pack("<d", 1.5)
    assert _read_constant(num, 0) == (1.5, 9)

    text = "abc".encode("utf-8")
    s = bytes([5]) + struct.pack("<I", len(text)) + text
    assert _read_constant(s, 0) == ("abc", 8)

    with pytest.raises(ValueError):
        _read_constant(bytes([9]), 0)


def test_consts_read_constants_split_and_index_param():
    c1 = bytes([1])
    c2 = bytes([3]) + struct.pack("<i", 8)
    assert read_constants(c1 + c2) == (True, 8.0)

    code_data = b"ABCD"
    const_data = b"XYZ"
    chunk_size = 12 + len(code_data) + len(const_data)
    chunk = (
        struct.pack("<I", chunk_size)
        + struct.pack("<I", len(code_data))
        + code_data
        + struct.pack("<I", len(const_data))
        + const_data
    )
    got_const, got_code = split_chunk(chunk)
    assert got_const == const_data
    assert got_code == code_data

    with pytest.raises(ValueError):
        split_chunk(b"short")
    with pytest.raises(ValueError):
        split_chunk(struct.pack("<I", 9999) + b"\x00" * 20)

    assert read_param(bytes([5]), 0, False) == (5, 1)
    assert read_param(struct.pack("<I", 300), 0, True) == (300, 4)
    assert read_index(struct.pack("<b", -7), 0, False) == (-7, 1)
    assert read_index(struct.pack("<i", -300), 0, True) == (-300, 4)


def test_diagnostic():
    Diagnostic._cache.clear()
    d = Diagnostic(start_line=1, start_column=2, end_line=3, end_column=4, code=12000)
    assert d.level == "SourceMap"
    assert (
        repr(d)
        == "Diagnostic(code=12000, level=SourceMap, name=SourceMap, start=(1, 2), end=(3, 4))"
    )
    assert d.message == "Source map entry"

    Diagnostic._cache.clear()
    unknown = Diagnostic(
        start_line=1, start_column=1, end_line=1, end_column=1, code=65535
    )
    assert unknown.level == "Unknown"
    assert (
        repr(unknown)
        == "Diagnostic(code=65535, level=Unknown, name=65535, start=(1, 1), end=(1, 1))"
    )
    assert unknown.message == "Unknown diagnostic code"


def test_diagnostic_decode():
    diagnostics, source_map = decode_diagnostics(
        [1, 2, 3, 4, 1001, 10, 11, 12, 13, 12000]
    )
    assert len(diagnostics) == 1
    assert len(source_map) == 1
    assert isinstance(diagnostics[0], Diagnostic)
    assert (
        str(diagnostics[0]) == "[Error] 发生未知内部错误 (InternalError) at 1:2 - 3:4"
    )
    assert isinstance(source_map[0], SourceMapEntry)
    assert (
        repr(source_map[0])
        == "SourceMapEntry(start_line=10, start_column=11, end_line=12, end_column=13)"
    )


def test_opcode_immutability_and_names():
    with pytest.raises(AttributeError):
        OpCode.NEW = 1
    with pytest.raises(AttributeError):
        del OpCode.ADD
    with pytest.raises(AttributeError):
        _ = OpCode.NO_SUCH

    assert get_opcode_name(-99999) == "-99999"

    public_names = [k for k in OpCode.__dict__.keys() if not k.startswith("_")]
    assert public_names
    assert OpCode.__getattr__(public_names[0]) == OpCode.__dict__[public_names[0]]


def test_wrap_vm_script_exception_branch():
    wrapped = wrap_vm_script(
        RuntimeError("x"), filename="f", source="s", ast=None, input_mode="script"
    )
    with pytest.raises(RuntimeError):
        wrapped()
    assert wrapped.filename == "f"
    assert wrapped.source == "s"
    assert wrapped.ast is None
    assert wrapped.input_mode == "script"
