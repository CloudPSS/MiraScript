"""pytest 共享 fixtures 与配置。"""

from __future__ import annotations

import logging
import sys
from pathlib import Path
from typing import Any

import pytest

from mirascript import (
    VmError,
    VmModule,
    compile as mira_compile,
    config_checkpoint,
    vm_function,
)
from .deepequals import assert_deep_equal, assert_not_deep_equal

# Mira 测试用例目录
TEST_DIR = (Path(__file__) / "../../../../tests").resolve()

# 要跳过的测试文件列表（相对于 TEST_DIR），使用 POSIX 路径格式（即使在 Windows 上），不添加 "./" 前缀
# 如 "e2e/complex.mira" 表示 TEST_DIR/e2e/complex.mira
SKIP_TESTS: set[str] = set({})


def _collect_mira_files() -> list[Path]:
    """收集所有 .mira 测试文件。"""
    return sorted(TEST_DIR.rglob("*.mira"))


def _make_vm_helpers() -> dict[str, Any]:
    """创建注入 Mira 脚本的全局辅助函数与变量。"""

    timeout_fns: list[tuple[Any, str | None]] = []

    @vm_function
    def t_eq(a, b, message=None):
        logging.debug("t_eq: %s == %s", a, b)
        assert_deep_equal(a, b, message=message)

    @vm_function
    def t_ne(a, b, message=None):
        logging.debug("t_ne: %s != %s", a, b)
        assert_not_deep_equal(a, b, message=message)

    @vm_function
    def t_true(v, message=None):
        logging.debug("t_true: %s", v)
        assert v, message

    @vm_function
    def t_false(v, message=None):
        logging.debug("t_false: %s", v)
        assert not v, message

    @vm_function
    def t_throws(fn, message=None):
        logging.debug("t_throws: expecting exception from %s", fn)
        try:
            fn()
        except VmError:
            return
        msg = message or "Expected VmError but none was raised"
        raise AssertionError(msg)

    @vm_function
    def t_timeout(fn, message=None):
        logging.debug("t_timeout: expecting timeout from %s", fn)
        timeout_fns.append(
            (fn, message if message is not None else "Execution timed out")
        )

    @vm_function
    def t_never(message=None):
        logging.debug("t_never: this should never be called")
        msg = message or "This should never be called"
        raise AssertionError(msg)

    return {
        "t_eq": t_eq,
        "t_ne": t_ne,
        "t_true": t_true,
        "t_false": t_false,
        "t_throws": t_throws,
        "t_timeout": t_timeout,
        "t_never": t_never,
        "v_array": [],
        "v_record": {},
        "v_nil": None,
        "v_true": True,
        "v_false": False,
        "v_number": 42,
        "v_string": "Hello, Mira!",
        "v_fn": vm_function(lambda: "I am a function"),
        "v_fn_another": vm_function(lambda: "I am another function"),
        "has_extern": False,
        "v_module": VmModule("v_module", {}),
        "v_module_another": VmModule("v_module_another", {}),
        "_timeout_fns": timeout_fns,
    }


@pytest.fixture(scope="session", autouse=True)
def _setup_mirascript():
    """全局初始化 MiraScript 环境。"""
    logging.basicConfig(level=logging.DEBUG)
    sys.setrecursionlimit(10000)
    config_checkpoint(10000)
    if mira_compile is None:
        pytest.skip("mirascript Python API not available")


@pytest.fixture
def vm_helpers() -> dict[str, Any]:
    """提供注入 Mira 脚本的辅助函数（每个测试独立）。"""
    return _make_vm_helpers()


def pytest_generate_tests(metafunc: pytest.Metafunc):
    """动态生成黑盒测试参数：为每个 .mira 文件生成一条用例。"""
    if "mira_file" in metafunc.fixturenames:
        files = _collect_mira_files()
        params = []
        ids = []
        for f in files:
            rel = f.relative_to(TEST_DIR).as_posix()
            test_id = (
                rel.replace(".mira", "")
                .replace("/", "_")
                .replace(".", "_")
                .replace("-", "_")
            )
            marks = []
            if rel in SKIP_TESTS:
                marks.append(
                    pytest.mark.skip(reason="Test skipped due to known issues")
                )
            params.append(pytest.param(f, id=test_id, marks=marks))
            ids.append(test_id)
        metafunc.parametrize("mira_file", params, ids=ids)
