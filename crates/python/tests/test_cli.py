"""
MiraScript CLI (__main__.py) 测试

测试 ``python -m mirascript`` 命令行接口的各种调用方式。
"""

from __future__ import annotations

import ast
import sys
import types
from io import StringIO
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import pytest

import mirascript.__main__ as cli

# 示例 .mira 文件目录
EXAMPLES_DIR = (Path(__file__).parent / "../../../examples").resolve()


# ---------------------------------------------------------------------------
# 工具函数
# ---------------------------------------------------------------------------


def _run_main(
    args: list[str], *, prog: str | None = "mirascript", stdin: str | None = None
) -> tuple[int, str, str]:
    """运行 main 函数，返回 (exit_code, stdout, stderr)。"""

    stdout = StringIO()
    stderr = StringIO()

    @patch("sys.argv", [prog or "mirascript", *args])
    @patch("sys.stdout", stdout)
    @patch("sys.stderr", stderr)
    @patch("sys.stdin", StringIO(stdin) if stdin is not None else StringIO())
    def run_main():
        return cli.main(prog=prog)

    exit_code = run_main()
    return exit_code, stdout.getvalue(), stderr.getvalue()


# ---------------------------------------------------------------------------
# 无参数 / 帮助
# ---------------------------------------------------------------------------


def test_no_args_shows_help():
    """不带任何参数时应打印帮助并返回 1。"""
    exit_code, stdout, stderr = _run_main([])
    assert exit_code == 1
    assert "usage:" in stdout or "usage:" in stderr


def test_no_args_when_prog_is_none():
    """prog 为 None（即 __name__ == '__main__' 路径）时也能正常显示帮助。"""
    exit_code, stdout, stderr = _run_main([], prog=None)
    assert exit_code == 1
    assert "usage:" in stdout or "usage:" in stderr


# ---------------------------------------------------------------------------
# --eval 模式
# ---------------------------------------------------------------------------


def test_eval_basic():
    """通过 --eval 执行简单表达式。"""
    exit_code, stdout, stderr = _run_main(["-e", "return 42;"])
    assert exit_code == 0
    assert "[OK] 42" in stdout


def test_eval_string():
    """通过 --eval 执行字符串表达式。"""
    exit_code, stdout, stderr = _run_main(["-e", 'return "hello";'])
    assert exit_code == 0
    assert stdout == "[OK] 'hello'\n"


def test_eval_arithmetic():
    """通过 --eval 执行算术表达式。"""
    exit_code, stdout, stderr = _run_main(["-e", "return 1 + 2 * 3;"])
    assert exit_code == 0
    assert stdout == "[OK] 7\n"


def test_eval_with_prog_none():
    """__main__ 直接调用时 --eval 也能正常工作。"""
    exit_code, stdout, stderr = _run_main(["-e", "return 99;"], prog=None)
    assert exit_code == 0
    assert stdout == "[OK] 99\n"


# ---------------------------------------------------------------------------
# --eval 与 --variable
# ---------------------------------------------------------------------------


def test_eval_with_single_variable():
    """通过 -v 传入单个变量。"""
    exit_code, stdout, stderr = _run_main(["-v", "x=10", "-e", "return x;"])
    assert exit_code == 0
    assert stdout == "[OK] 10\n"


def test_eval_with_multiple_variables():
    """通过多个 -v 传入多个变量。"""
    exit_code, stdout, stderr = _run_main(
        ["-v", "a=3", "-v", "b=4", "-e", "return a + b;"]
    )
    assert exit_code == 0
    assert stdout == "[OK] 7\n"


def test_eval_with_string_variable():
    """传入字符串变量，通过插值使用。"""
    exit_code, stdout, stderr = _run_main(
        ["-v", 'name="Mira"', "-e", 'return "Hello, $name!";']
    )
    assert exit_code == 0
    assert stdout == "[OK] 'Hello, Mira!'\n"


def test_eval_with_expression_variable():
    """变量值可以是表达式。"""
    exit_code, stdout, stderr = _run_main(["-v", "x=3*4", "-e", "return x;"])
    assert exit_code == 0
    assert stdout == "[OK] 12\n"


def test_invalid_variable_format():
    """无效的变量定义格式应报错。"""
    exit_code, stdout, stderr = _run_main(["-v", "no_equals_sign", "-e", "return 1;"])
    assert exit_code == 1
    assert "Invalid variable definition" in stderr


def test_invalid_variable_value():
    """变量值编译失败应报错。"""
    exit_code, stdout, stderr = _run_main(["-v", "x=!!!", "-e", "return 1;"])
    assert exit_code == 1


# ---------------------------------------------------------------------------
# 脚本文件执行
# ---------------------------------------------------------------------------


def test_run_mira_file():
    """执行一个 .mira 脚本文件。"""
    hello_file = EXAMPLES_DIR / "01_hello_world.mira"
    exit_code, stdout, stderr = _run_main([str(hello_file)])
    # hello_world.mira 没有 return 语句，结果可能是 nil
    assert exit_code == 0


def test_run_mira_file_41_fib():
    """执行 fib 示例确认返回值。"""
    fib_file = EXAMPLES_DIR / "41_fib.mira"
    exit_code, stdout, stderr = _run_main([str(fib_file)])
    assert exit_code == 0


def test_nonexistent_file():
    """执行不存在的文件应报错。"""
    exit_code, stdout, stderr = _run_main(["nonexistent_file.mira"])
    assert exit_code == 1
    assert "does not exist" in stderr or "Error" in stderr


# ---------------------------------------------------------------------------
# stdin 输入
# ---------------------------------------------------------------------------


def test_stdin_input():
    """通过 stdin 传入代码。"""
    exit_code, stdout, stderr = _run_main(["-"], stdin="return 123;")
    assert exit_code == 0
    assert stdout == "[OK] 123\n"


# ---------------------------------------------------------------------------
# --template 模式
# ---------------------------------------------------------------------------


def test_template_mode_basic():
    """--template 模式执行模板，变量通过 -v 传入。"""
    exit_code, stdout, stderr = _run_main(
        ["-v", 'name="World"', "-t", "-e", '"Hello, $name!"']
    )
    assert exit_code == 0
    assert "Hello, World!" in stdout


# ---------------------------------------------------------------------------
# --generate 模式
# ---------------------------------------------------------------------------


def test_generate_output_file():
    """--generate 将生成的 Python 代码写入文件。"""
    with TemporaryDirectory() as tmpdir:
        output = Path(tmpdir) / "output.py"
        exit_code, stdout, stderr = _run_main(["-g", str(output), "-e", "return 42;"])
        assert exit_code == 0
        assert output.is_file()
        content = output.read_text(encoding="utf-8")
        assert "def " in content or "script" in content.lower()


def test_generate_output_file_with_variables():
    """--generate 与变量一起使用。"""
    with TemporaryDirectory() as tmpdir:
        output = Path(tmpdir) / "output2.py"
        exit_code, stdout, stderr = _run_main(
            ["-v", "x=7", "-g", str(output), "-e", "return x * 6;"]
        )
        assert exit_code == 0
        assert "[OK] 42" in stdout
        assert output.is_file()


# ---------------------------------------------------------------------------
# 边界 / 错误场景
# ---------------------------------------------------------------------------


def test_eval_and_file_together():
    """同时使用 --eval 和文件参数应报错。"""
    exit_code, stdout, stderr = _run_main(["-e", "return 1;", "some_file.mira"])
    assert exit_code == 1
    assert "cannot be used with" in stderr.lower() or "Error" in stderr


def test_eval_syntax_error():
    """语法错误的代码应返回非 0 退出码。"""
    exit_code, stdout, stderr = _run_main(["-e", "this is not valid code !!!"])
    assert exit_code == 1


def test_eval_with_no_return():
    """没有 return 语句的 eval（模板模式测试）。"""
    exit_code, stdout, stderr = _run_main(["-t", "-e", "just a string"])
    assert exit_code == 0


# ---------------------------------------------------------------------------
# 白盒：__main__ 内部函数与异常路径
# ---------------------------------------------------------------------------


def test_compile_exception_exits_with_code_2():
    with patch.object(cli, "compile", side_effect=RuntimeError("boom")):
        with pytest.raises(SystemExit) as exc:
            cli._compile("return 1;", "script")
    assert exc.value.code == 2


if sys.version_info >= (3, 9):

    def test_get_unparse_prefers_stdlib():
        unparse = cli._get_unparse()
        assert unparse is ast.unparse

    def test_get_unparse_raises_if_all_missing(monkeypatch: pytest.MonkeyPatch):
        monkeypatch.delattr(ast, "unparse", raising=False)
        monkeypatch.delitem(sys.modules, "astunparse", raising=False)

        with pytest.raises(ImportError):
            cli._get_unparse()


def test_get_unparse_fallback_to_astunparse(monkeypatch: pytest.MonkeyPatch):
    monkeypatch.delattr(ast, "unparse", raising=False)

    fake = types.ModuleType("astunparse")

    def fake_unparse(node):
        return "fake_unparse"

    fake.unparse = fake_unparse  # pyright: ignore[reportAttributeAccessIssue]
    monkeypatch.setitem(sys.modules, "astunparse", fake)

    unparse = cli._get_unparse()
    assert unparse is fake_unparse


def test_print_debug_writes_python_file(tmp_path: Path):
    script, diagnostics = cli.compile(
        "return 42;", input_mode="script", filename="<test>"
    )
    assert diagnostics is not None
    assert script is not None

    out = tmp_path / "debug_out.py"
    cli._print_debug(script, str(out), {"x": 1})

    content = out.read_text(encoding="utf-8")
    assert "if __name__ == '__main__'" in content
    assert "result = script({'x': 1})" in content


def test_main_returns_1_when_script_raises():
    def bad_script(_ctx=None):
        raise RuntimeError("runtime failure")

    with patch.object(cli, "_compile", return_value=(bad_script, [])):
        exit_code, stdout, stderr = _run_main(["-e", "return 1;"])

    assert exit_code == 1


def test_main_variable_evaluation_exception_returns_1():
    def compile_side_effect(code, mode, filename=None):
        if filename and filename.startswith("<variable:"):
            raise RuntimeError("var compile failure")
        return cli.compile(code, input_mode=mode, filename=filename)

    with patch.object(cli, "_compile", side_effect=compile_side_effect):
        exit_code, stdout, stderr = _run_main(["-v", "x=1", "-e", "return 1;"])

    assert exit_code == 1
    assert "Error evaluating variable" in stderr
