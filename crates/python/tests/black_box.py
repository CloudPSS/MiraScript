"""
MiraScript 黑盒测试

遍历 tests/ 下所有 .mira 文件，注入辅助函数后在 VM 中执行。
每个 .mira 文件对应一个参数化测试用例。
"""

from __future__ import annotations
from typing_extensions import TYPE_CHECKING

from mirascript import vm_function, VmError, VmModule, VmValue, VmFunction
from .deepequals import deep_equal

if TYPE_CHECKING:
    from typing_extensions import Callable, TypeAlias, List, Tuple, Dict

    TimeoutFn: TypeAlias = Callable[[], None]
    TimeoutFns: TypeAlias = List[Tuple[TimeoutFn, str]]
    VmTestHelpers: TypeAlias = Tuple[TimeoutFns, Dict[str, VmValue]]


@vm_function
def t_eq(a: VmValue, b: VmValue, message: str | None = None):
    assert deep_equal(a, b), message


@vm_function
def t_ne(a: VmValue, b: VmValue, message: str | None = None):
    assert not deep_equal(a, b), message


@vm_function
def t_true(v: VmValue, message: str | None = None):
    assert v is True, message


@vm_function
def t_false(v: VmValue, message: str | None = None):
    assert v is False, message


@vm_function
def t_throws(fn: VmFunction, message: str | None = None):
    try:
        fn()
    except VmError:
        return
    msg = message or "Expected VmError but none was raised"
    raise AssertionError(msg)


@vm_function
def t_never(message: str = "This should never be called"):
    raise AssertionError(message)


def make_vm_helpers() -> VmTestHelpers:
    """创建注入 Mira 脚本的全局辅助函数与变量。"""

    timeout_fns: TimeoutFns = []

    @vm_function
    def t_timeout(fn: TimeoutFn, message: str = "Execution timed out"):
        timeout_fns.append((fn, message))

    context = {
        "t_eq": t_eq,
        "t_ne": t_ne,
        "t_true": t_true,
        "t_false": t_false,
        "t_throws": t_throws,
        "t_never": t_never,
        "t_timeout": t_timeout,
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
    }

    return timeout_fns, context
