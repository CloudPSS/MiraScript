"""VmSharedContext 快照测试。

验证共享上下文中注册的全局键是否与预期一致。
"""

from mirascript._vm.types.context import get_shared_context


def test_shared_context_keys(snapshot):
    """VmSharedContext 应包含预期的全局键。"""
    keys = sorted(get_shared_context().keys())
    snapshot.assert_match(repr(keys), "shared_context_keys.txt")
