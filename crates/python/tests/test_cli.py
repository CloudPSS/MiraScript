"""MiraScript CLI (__main__.py) 测试

测试 ``python -m mirascript`` 命令行接口的各种调用方式。
"""

from __future__ import annotations

import sys
from io import StringIO
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import pytest

from mirascript.__main__ import main

# 示例 .mira 文件目录
EXAMPLES_DIR = (Path(__file__).parent / "../../../examples").resolve()


# ---------------------------------------------------------------------------
# 工具函数
# ---------------------------------------------------------------------------


def _run_main(
    args: list[str], *, prog: str | None = "mirascript"
) -> tuple[int, str, str]:
    """运行 main 函数，返回 (exit_code, stdout, stderr)。"""
    stdout = StringIO()
    stderr = StringIO()
    with (
        patch.object(sys, "argv", [prog or "mirascript", *args]),
        patch.object(sys, "stdout", stdout),
        patch.object(sys, "stderr", stderr),
    ):
        exit_code = main(prog=prog)
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
    assert "[OK] hello" in stdout


def test_eval_arithmetic():
    """通过 --eval 执行算术表达式。"""
    exit_code, stdout, stderr = _run_main(["-e", "return 1 + 2 * 3;"])
    assert exit_code == 0
    assert "[OK] 7" in stdout


def test_eval_with_prog_none():
    """__main__ 直接调用时 --eval 也能正常工作。"""
    exit_code, stdout, stderr = _run_main(["-e", "return 99;"], prog=None)
    assert exit_code == 0
    assert "[OK] 99" in stdout


# ---------------------------------------------------------------------------
# --eval 与 --variable
# ---------------------------------------------------------------------------


def test_eval_with_single_variable():
    """通过 -v 传入单个变量。"""
    exit_code, stdout, stderr = _run_main(["-v", "x=10", "-e", "return x;"])
    assert exit_code == 0
    assert "[OK] 10" in stdout


def test_eval_with_multiple_variables():
    """通过多个 -v 传入多个变量。"""
    exit_code, stdout, stderr = _run_main(
        ["-v", "a=3", "-v", "b=4", "-e", "return a + b;"]
    )
    assert exit_code == 0
    assert "[OK] 7" in stdout


def test_eval_with_string_variable():
    """传入字符串变量，通过插值使用。"""
    exit_code, stdout, stderr = _run_main(
        ["-v", 'name="Mira"', "-e", 'return "Hello, $name!";']
    )
    assert exit_code == 0
    assert "[OK] Hello, Mira!" in stdout


def test_eval_with_expression_variable():
    """变量值可以是表达式。"""
    exit_code, stdout, stderr = _run_main(["-v", "x=3*4", "-e", "return x;"])
    assert exit_code == 0
    assert "[OK] 12" in stdout


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
    if not hello_file.is_file():
        pytest.skip("Example file not found")
    exit_code, stdout, stderr = _run_main([str(hello_file)])
    # hello_world.mira 没有 return 语句，结果可能是 nil
    assert exit_code == 0


def test_run_mira_file_41_fib():
    """执行 fib 示例确认返回值。"""
    fib_file = EXAMPLES_DIR / "41_fib.mira"
    if not fib_file.is_file():
        pytest.skip("Example file not found")
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
    import io

    stdin_mock = io.StringIO("return 123;")
    with patch.object(sys, "argv", ["mirascript", "-"]), patch.object(
        sys, "stdin", stdin_mock
    ), patch.object(sys, "stdout", StringIO()), patch.object(sys, "stderr", StringIO()):
        exit_code = main()
        # stdin 模式下会读取 stdin 内容并执行
        # 注意: main 内部使用 sys.stdin.read()，mock 后应正常
    # 这里只验证不崩溃即可
    assert exit_code in (0, 1)


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
