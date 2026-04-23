import logging
import unittest
import sys
from pathlib import Path
from mirascript.main import compile as mira_compile
from mirascript.vm import VmError, VmFunction, VmModule
from mirascript.vm.types.context import create_vm_context
from mirascript.vm.helpers import config_checkpoint
from mirascript.mirascript import Config
from .deepequals import assert_deep_equal, assert_not_deep_equal

TEST_DIR = (Path(__file__) / "../../../../tests").resolve()

SKIP_TESTS = {
    "e2e/complex.mira",
    "lib/sequence/sort_by.mira",
    "lib/sequence/sort.mira",
    "lib/math/gamma.mira",
    "lib/string/case.mira",
    "lib/sequence/unique_by.mira",
    "lib/sequence/unique.mira",
    "lib/sequence/new.mira",
    "logic/loop.mira",
    "feature/module.mira",
}


class BlackBoxTests(unittest.TestCase):
    def setUp(self) -> None:
        sys.setrecursionlimit(10000)
        if mira_compile is None:
            self.skipTest("mirascript Python API not available")

    def run_mira_file(self, mira_path: Path):
        code = mira_path.read_text(encoding="utf-8")
        expected_path = Path(str(mira_path) + ".jsonl")
        expected = (
            expected_path.read_text(encoding="utf-8")
            if expected_path.exists()
            else None
        )

        # 收集超时回调与脚本输出
        timeout_fns = []

        def t_eq(a, b, message=None):
            logging.debug(f"t_eq: {a} == {b}")
            assert_deep_equal(a, b, message=message)

        def t_ne(a, b, message=None):
            logging.debug(f"t_ne: {a} != {b}")
            assert_not_deep_equal(a, b, message=message)

        def t_true(v, message=None):
            self.assertTrue(v, msg=message)

        def t_false(v, message=None):
            self.assertFalse(v, msg=message)

        def t_throws(fn, message=None):
            logging.debug(f"t_throws: expecting exception from {fn}")
            with self.assertRaises(VmError, msg=message):
                fn()

        def t_timeout(fn, message=None):
            timeout_fns.append(
                [fn, message if message is not None else "Execution timed out"]
            )

        def t_never(message=None):
            msg = message or "This should never be called"
            self.fail(msg)

        # environment values passed to script
        externs = {
            "t_eq": VmFunction(t_eq),
            "t_ne": VmFunction(t_ne),
            "t_true": VmFunction(t_true),
            "t_false": VmFunction(t_false),
            "t_throws": VmFunction(t_throws),
            "t_timeout": VmFunction(t_timeout),
            "t_never": VmFunction(t_never),
            "v_array": [],
            "v_record": {},
        }
        globals_ = {
            "v_nil": None,
            "v_true": True,
            "v_false": False,
            "v_number": 42,
            "v_string": "Hello, Mira!",
            "v_fn": VmFunction(lambda: "I am a function"),
            "v_fn_another": VmFunction(lambda: "I am another function"),
            "has_extern": True,
            "v_extern": {},
            "v_extern_another": {},
            "has_module": True,
            "v_module": VmModule("v_module", {}),
            "v_module_another": VmModule("v_module_another", {}),
        }

        # adjust checkpoint for huge tests (mimic TS behavior)
        if mira_path.name.endswith("_huge.mira"):
            config_checkpoint(5000)
        else:
            config_checkpoint(1000)

        # compile and run
        script, x = mira_compile(
            code,
            Config(
                **{"pretty": True, "sourceMap": True, "fileName": mira_path.as_uri()}
            ),
        )
        ctx = create_vm_context(externs, globals_)
        # script 应当是可调用的：script(ctx)
        script(ctx)

        # run timeout callbacks after script execution
        for [fn, message] in timeout_fns:
            with self.assertRaisesRegex(
                RuntimeError, "Execution timed out", msg=message
            ):
                fn()

    def test(self):
        files = TEST_DIR.rglob("*.mira")

        for mira_path in files:
            file = mira_path.relative_to(TEST_DIR).as_posix()
            if file in SKIP_TESTS:
                logging.debug(f"Skipping test {file}")
                continue
            with self.subTest(file=file):
                self.run_mira_file(mira_path)


if __name__ == "__main__":
    logging.basicConfig(level=logging.DEBUG)
    unittest.main()
