import math

import pytest

from mirascript import compile
from mirascript._vm.operations.cp import config_checkpoint, thread_local


def test_config_checkpoint():
    with pytest.raises(ValueError, match="Invalid timeout value"):
        config_checkpoint(timeout=-1)
    with pytest.raises(ValueError, match="Invalid timeout value"):
        config_checkpoint(timeout=True)
    with pytest.raises(ValueError, match="Invalid timeout value"):
        config_checkpoint(timeout=float("nan"))
    with pytest.raises(ValueError, match="Invalid timeout value"):
        config_checkpoint(timeout=float("inf"))

    with pytest.raises(ValueError, match="Invalid check interval value"):
        config_checkpoint(check_interval=0)
    with pytest.raises(ValueError, match="Invalid check interval value"):
        config_checkpoint(check_interval=0.0)  # type: ignore


def test_stack_overflow():
    config_checkpoint()

    assert thread_local.cp_depth == 0
    assert thread_local.cp_counter == 0
    assert math.isnan(thread_local.cp)

    script, _ = compile("""
    fn f {
      f();
    }
    f();
    """)
    assert script is not None
    with pytest.raises(RuntimeError, match="Maximum call depth exceeded"):
        script()
    with pytest.raises(RuntimeError, match="Maximum call depth exceeded"):
        script()
    with pytest.raises(RuntimeError, match="Maximum call depth exceeded"):
        script()

    assert thread_local.cp_depth == 0
    assert thread_local.cp_counter == 0
    assert math.isnan(thread_local.cp)

    # VM should not corrupted after stack overflow
    script, _ = compile("[1, 2, 3]::map(fn { it + x })")
    assert script is not None
    result = script({"x": 4})
    assert result == [5, 6, 7]
