import traceback
import sys
import astor
from pathlib import Path

import mirascript

if __name__ == "__main__":
    # Read script file from arg
    if len(sys.argv) < 2:
        print("Usage: python debug.py <script_file.mira>")
        sys.exit(1)

    if sys.argv[1] == "-":
        script = sys.stdin.read()
    else:
        script_file = Path(sys.argv[1]).resolve()
        if not script_file.is_file():
            print(f"Error: File '{script_file}' does not exist.")
            sys.exit(1)
        script = script_file.read_text(encoding="utf-8")

    mode = "template" if sys.argv[1].endswith(".miratpl") else "script"
    # Compile and execute the script
    try:
        config = mirascript.Config(input_mode=mode)
        result, diagnostics = mirascript.compile(script, config)
        for diag in diagnostics:
            print(diag)
        if result is None:
            sys.exit(1)
        assert callable(result), "Compilation failed, result is not callable"
        # Write the result to a file for debugging
        with open("debug_result.py", "w", encoding="utf-8") as f:
            f.write(
                "from mirascript.vm.helpers import *\n"
                "from mirascript.vm.operations import *\n"
                "from mirascript.vm.types.const import *\n"
                "\n"
                f"{astor.to_source(result.__module__)}"
                "\n"
                "if __name__ == '__main__':\n"
                "    result = script()\n"
                "    print('[OK]', result)"
            )

        print("[OK]", result())
    except Exception as e:
        traceback.print_exc()
        print(f"Error during compilation or execution: {e}")
