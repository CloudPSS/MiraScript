import unittest
import os
from pathlib import Path

from mirascript.main import compile as mira_compile
from mirascript.vm import VmError, VmFunction, VmModule
from mirascript.vm.types.context import create_vm_context
from mirascript.vm.helpers import config_checkpoint
from mirascript.mirascript import Config
from deepequals import assert_deep_equal,assert_not_deep_equal
from mirascript.vm.lib.vm_global.json_ import NanToNullEncoder
TEST_DIR = Path(__file__).resolve().parents[3] / "tests"  # 对应 ts 中 ../../../tests


class BlackBoxTests(unittest.TestCase):
    def setUp(self) -> None:
        if mira_compile is None:
            self.skipTest("mirascript Python API not available")

    def run_mira_file(self, mira_path: Path):
        code = mira_path.read_text(encoding="utf-8")
        expected_path = Path(str(mira_path) + ".jsonl")
        expected = expected_path.read_text(encoding="utf-8") if expected_path.exists() else None

        # 收集超时回调与脚本输出
        timeout_fns = []
        result_lines = []

        # helper wrappers that call unittest assertions immediately
        def t_eq(a, b,message=None):
            print(f"t_eq start: {a} == {b},type a {type(a)}, type b {type(b)}")  # --- DEBUG ---
            assert_deep_equal(a, b,message=message)
            print(f"t_eq end: {a} == {b},type a {type(a)}, type b {type(b)}")  # --- DEBUG ---

        def t_ne(a, b,message=None):
            # self.assertNotEqual(a, b)
            assert_not_deep_equal(a, b,message=message)
            

        def t_true(v,message=None):
            print(f"t_true: {v}, type {type(v)}")  # --- DEBUG ---
            self.assertTrue(v,msg=message)
        def t_false(v,message=None):
            print(f"t_false: {v}, type {type(v)}")  # --- DEBUG ---
            self.assertFalse(v,msg=message)

        def t_throws(fn,message=None):
            print('t_throws',fn,type(fn))  # --- DEBUG ---
            with self.assertRaises(VmError, msg=message):
                fn()
            print('t_throws end',fn,type(fn))  # --- DEBUG ---
        def t_timeout(fn,message=None):
            print(f"t_timeout: registering timeout function {fn}")  # --- DEBUG ---
            timeout_fns.append([fn, message if  message is not None else "Execution timeout"])

        def t_snapshot(*values):
            import json
            print("t_snapshot:",values)  # --- DEBUG ---
            result_lines.append(json.dumps(list(values), cls=NanToNullEncoder,ensure_ascii=False) + "\n")

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
            "t_snapshot": VmFunction(t_snapshot),
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
            config_checkpoint(1000)
        else:
            config_checkpoint(1000)

        # compile and run
        script,x = mira_compile(code, Config(**{"pretty": True, "sourceMap": True, "fileName": mira_path.as_uri()}))
        ctx = create_vm_context(externs, globals_)
        # script 应当是可调用的：script(ctx)
        script(ctx)

        # run timeout callbacks after script execution (mimic TS)
        print(f"timeout_fns count: {len(timeout_fns)}")  # --- DEBUG ---
        for [fn, message] in timeout_fns:
            with self.assertRaisesRegex(RangeError if 'RangeError' in globals() else Exception,message):
                fn()

       
    def test_all_mira_files(self):
        # print(len(sorted(TEST_DIR.rglob("*.mira"))))
        files =sorted(TEST_DIR.rglob("*.mira"))
        
        for mira_path in files:
                print(files.index(mira_path),len(files))  # --- DEBUG ---
                # if  'lib' in mira_path.parts:
                #     # 这个测试文件目前有问题，跳过
                #     continue
                print(f"Running test: {mira_path}")  # --- DEBUG ---
            # with self.subTest(mira_file=str(mira_path.relative_to(TEST_DIR))):
                self.run_mira_file(mira_path)
                # break

if __name__ == "__main__":
    unittest.main()