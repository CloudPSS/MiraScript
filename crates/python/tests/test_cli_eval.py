"""
MiraScript CLI (__main__.py) 测试

测试 ``python -m mirascript`` 命令行接口的各种调用方式。
"""

from __future__ import annotations
from pathlib import Path
from tempfile import TemporaryDirectory

from .cli import run_main

# ---------------------------------------------------------------------------
# --eval 模式
# ---------------------------------------------------------------------------


def test_eval_basic():
    """通过 --eval 执行简单表达式。"""
    exit_code, stdout, _ = run_main(["-e", "return 42;"])
    assert exit_code == 0
    assert "[OK] 42" in stdout


def test_eval_string():
    """通过 --eval 执行字符串表达式。"""
    exit_code, stdout, _ = run_main(["-e", 'return "hello";'])
    assert exit_code == 0
    assert stdout == "[OK] 'hello'\n"


def test_eval_arithmetic():
    """通过 --eval 执行算术表达式。"""
    exit_code, stdout, _ = run_main(["-e", "return 1 + 2 * 3;"])
    assert exit_code == 0
    assert stdout == "[OK] 7\n"


# ---------------------------------------------------------------------------
# --eval 与 --variable
# ---------------------------------------------------------------------------


def test_eval_with_single_variable():
    """通过 -v 传入单个变量。"""
    exit_code, stdout, _ = run_main(["-v", "x=10", "-e", "return x;"])
    assert exit_code == 0
    assert stdout == "[OK] 10\n"


def test_eval_with_multiple_variables():
    """通过多个 -v 传入多个变量。"""
    exit_code, stdout, _ = run_main(["-v", "a=3", "-v", "b=4", "-e", "return a + b;"])
    assert exit_code == 0
    assert stdout == "[OK] 7\n"


def test_eval_with_string_variable():
    """传入字符串变量，通过插值使用。"""
    exit_code, stdout, _ = run_main(
        ["-v", 'name="Mira"', "-e", 'return "Hello, $name!";']
    )
    assert exit_code == 0
    assert stdout == "[OK] 'Hello, Mira!'\n"


def test_eval_with_expression_variable():
    """变量值可以是表达式。"""
    exit_code, stdout, _ = run_main(["-v", "x=3*4", "-e", "return x;"])
    assert exit_code == 0
    assert stdout == "[OK] 12\n"


def test_invalid_variable_format():
    """无效的变量定义格式应报错。"""
    exit_code, _, stderr = run_main(["-v", "no_equals_sign", "-e", "return 1;"])
    assert exit_code == 1
    assert "Invalid variable definition" in stderr


def test_invalid_variable_value():
    """变量值编译失败应报错。"""
    exit_code, _, _ = run_main(["-v", "x=!!!", "-e", "return 1;"])
    assert exit_code == 1


# ---------------------------------------------------------------------------
# --template 模式
# ---------------------------------------------------------------------------


def test_template_mode_basic():
    """--template 模式执行模板，变量通过 -v 传入。"""
    exit_code, stdout, _ = run_main(
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
        exit_code, _, _ = run_main(["-g", str(output), "-e", "return 42;"])
        assert exit_code == 0
        assert output.is_file()
        content = output.read_text(encoding="utf-8")
        assert "def " in content or "script" in content.lower()


def test_generate_output_file_with_variables():
    """--generate 与变量一起使用。"""
    with TemporaryDirectory() as tmpdir:
        output = Path(tmpdir) / "output2.py"
        exit_code, stdout, _ = run_main(
            ["-v", "x=7", "-g", str(output), "-e", "return x * 6;"]
        )
        assert exit_code == 0
        assert "[OK] 42" in stdout
        assert output.is_file()
