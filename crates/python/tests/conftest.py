"""pytest 共享 fixtures 与配置。"""

from __future__ import annotations

import logging
import sys
from pathlib import Path

import pytest

from mirascript import compile as mira_compile

# Mira 测试用例目录
TEST_DIR = (Path(__file__) / "../../../../tests").resolve()

# 要跳过的测试文件列表（相对于 TEST_DIR），使用 POSIX 路径格式（即使在 Windows 上），不添加 "./" 前缀
# 如 "e2e/complex.mira" 表示 TEST_DIR/e2e/complex.mira
SKIP_TESTS: set[str] = set({})


def _collect_mira_files() -> list[Path]:
    """收集所有 .mira 测试文件。"""
    return sorted(TEST_DIR.rglob("*.mira"))


@pytest.fixture(scope="session", autouse=True)
def _setup_mirascript():
    """全局初始化 MiraScript 环境。"""
    logging.basicConfig(level=logging.DEBUG)
    sys.setrecursionlimit(10000)
    if mira_compile is None:
        pytest.skip("mirascript Python API not available")


def pytest_generate_tests(metafunc: pytest.Metafunc):
    """动态生成黑盒测试参数：为每个 .mira 文件生成一条用例。"""
    if "mira_file" in metafunc.fixturenames:
        files = _collect_mira_files()
        params = []
        ids = []
        for f in files:
            rel = f.relative_to(TEST_DIR).as_posix()
            test_id = rel
            marks = []
            if rel in SKIP_TESTS:
                marks.append(
                    pytest.mark.skip(reason="Test skipped due to known issues")
                )
            params.append(pytest.param(f, id=test_id, marks=marks))
            ids.append(test_id)
        metafunc.parametrize("mira_file", params, ids=ids)
