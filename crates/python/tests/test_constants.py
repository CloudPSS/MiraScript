from mirascript import Uninitialized, VmUninitialized


def test_uninitialized_constant():
    """测试 Uninitialized 常量的类型和行为。"""

    # 检查类型
    assert Uninitialized is not None

    # 检查字符串表示
    assert str(Uninitialized) == "<uninitialized>"
    assert repr(Uninitialized) == "<uninitialized>"

    # 检查比较操作
    assert Uninitialized == Uninitialized
    assert not (Uninitialized != Uninitialized)

    # 检查哈希值
    assert hash(Uninitialized) == hash(Uninitialized)

    # 检查布尔值
    assert bool(Uninitialized) is False
