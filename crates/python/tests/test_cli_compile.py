"""
MiraScript CLI (__main__.py) 测试

测试 ``python -m mirascript`` 命令行接口的各种调用方式。
"""

from __future__ import annotations
from unittest.mock import patch
import pytest

import mirascript._main.compile as cli_compile
import mirascript._main.variables as cli_variables
import mirascript.__main__ as cli
import mirascript

from .cli import run_main

# ---------------------------------------------------------------------------
# 白盒：__main__ 内部函数与异常路径
# ---------------------------------------------------------------------------


def test_compile_exception_exits_with_code_2():
    with patch.object(cli_compile, "_compile", side_effect=RuntimeError("boom")):
        with pytest.raises(SystemExit) as exc:
            cli_compile.compile_code("return 1;", "script")
    assert exc.value.code == 2


def test_main_returns_1_when_script_raises():
    def bad_script(_=None):
        raise RuntimeError("runtime failure")

    with patch.object(cli, "compile_code", return_value=(bad_script, [])):
        exit_code, _, _ = run_main(["-e", "return 1;"])

    assert exit_code == 1


def test_main_variable_evaluation_exception_returns_1():
    def compile_side_effect(code, mode, filename=None):
        if filename and filename.startswith("<variable:"):
            raise RuntimeError("var compile failure")
        return mirascript.compile(code, input_mode=mode, filename=filename)

    with patch.object(cli_variables, "compile_code", side_effect=compile_side_effect):
        exit_code, _, stderr = run_main(["-v", "x=1", "-e", "return 1;"])

    assert exit_code == 1
    assert "Error evaluating variable" in stderr
