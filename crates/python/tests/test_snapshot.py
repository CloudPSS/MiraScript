"""VmSharedContext 快照测试。

验证共享上下文中注册的全局键是否与预期一致。
"""

from pytest_snapshot.plugin import Snapshot
from mirascript._vm.types.context import get_shared_context
import mirascript


def test_shared_context_keys(snapshot: Snapshot):
    """VmSharedContext 应包含预期的全局键。"""
    keys = sorted(get_shared_context().keys())
    snapshot.assert_match(repr(keys), "shared_context_keys.txt")


def test_module_exports(snapshot: Snapshot):
    """VmModule 应导出预期的属性。"""
    exports = sorted(name for name in dir(mirascript) if not name.startswith("_"))
    all = sorted(mirascript.__all__)
    assert exports == all, "导出的属性与 __all__ 不匹配"

    snapshot.assert_match(repr(exports), "exports.txt")
