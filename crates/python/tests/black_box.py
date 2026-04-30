import logging
import unittest
import sys
from pathlib import Path
from mirascript import compile as mira_compile
from mirascript.vm import VmError, vm_function, VmModule, VmContext
from mirascript.vm.helpers import config_checkpoint
from tests.deepequals import assert_deep_equal, assert_not_deep_equal

TEST_DIR = (Path(__file__) / "../../../../tests").resolve()

# 要跳过的测试文件列表（相对于 TEST_DIR），使用 POSIX 路径格式（即使在 Windows 上），不添加 "./" 前缀
# 如 "e2e/complex.mira" 表示 TEST_DIR/e2e/complex.mira
SKIP_TESTS = {
    "e2e/complex.mira",
    "feature/module.mira",
}


class BlackBoxTests(unittest.TestCase):
    def setUp(self) -> None:
        logging.basicConfig(level=logging.DEBUG)
        sys.setrecursionlimit(10000)
        if mira_compile is None:
            self.skipTest("mirascript Python API not available")

    def run_mira_file(self, mira_path: Path):
        code = mira_path.read_text(encoding="utf-8")

        # 收集超时回调与脚本输出
        timeout_fns = []

        @vm_function
        def t_eq(a, b, message=None):
            logging.debug(f"t_eq: {a} == {b}")
            assert_deep_equal(a, b, message=message)

        @vm_function
        def t_ne(a, b, message=None):
            logging.debug(f"t_ne: {a} != {b}")
            assert_not_deep_equal(a, b, message=message)

        @vm_function
        def t_true(v, message=None):
            logging.debug(f"t_true: {v}")
            self.assertTrue(v, msg=message)

        @vm_function
        def t_false(v, message=None):
            logging.debug(f"t_false: {v}")
            self.assertFalse(v, msg=message)

        @vm_function
        def t_throws(fn, message=None):
            logging.debug(f"t_throws: expecting exception from {fn}")
            with self.assertRaises(VmError, msg=message):
                fn()

        @vm_function
        def t_timeout(fn, message=None):
            logging.debug(f"t_timeout: expecting timeout from {fn}")
            timeout_fns.append(
                [fn, message if message is not None else "Execution timed out"]
            )

        @vm_function
        def t_never(message=None):
            logging.debug("t_never: this should never be called")
            msg = message or "This should never be called"
            self.fail(msg)

        # environment values passed to script
        globals_ = {
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
        }

        # adjust checkpoint for huge tests (mimic TS behavior)
        if mira_path.name.endswith("_huge.mira"):
            config_checkpoint(5000)
        else:
            config_checkpoint(1000)

        # compile and run
        script, _ = mira_compile(code, filename=mira_path)
        ctx = VmContext(globals_)
        # script 应当是可调用的：script(ctx)
        if script is None:
            self.fail("Compilation failed, no script generated")
        script(ctx)

        # run timeout callbacks after script execution
        for [fn, message] in timeout_fns:
            with self.assertRaisesRegex(
                RuntimeError, "Execution timed out", msg=message
            ):
                fn()


files = TEST_DIR.rglob("*.mira")

for test_file in files:
    file = test_file.relative_to(TEST_DIR).as_posix()
    test_name = "test_" + (
        file.replace(".mira", "").replace("/", "_").replace(".", "_").replace("-", "_")
    )
    test = (
        (lambda self: self.skipTest("Test skipped due to known issues"))
        if file in SKIP_TESTS
        else (lambda self, mira_path=test_file: self.run_mira_file(mira_path))
    )
    test.__doc__ = file
    setattr(BlackBoxTests, test_name, test)

if __name__ == "__main__":
    unittest.main()
