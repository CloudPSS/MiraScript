from pathlib import Path
import sys
import argparse
import traceback

from .compiler import compile, InputMode


def _compile(script: str, mode: InputMode):
    try:
        bytecode, diagnostics = compile(script, input_mode=mode)
        return bytecode, diagnostics
    except Exception as e:
        traceback.print_exc()
        print(f"Error during compilation: {e}")
        sys.exit(2)


def _print_debug(script, output_file):
    try:
        import astor

        with open(output_file, "w", encoding="utf-8") as f:
            f.write(
                "from mirascript.vm.helpers import *\n"
                "from mirascript.vm.operations import *\n"
                "from mirascript.vm.types.const import *\n"
                "\n"
                f"{astor.to_source(script.__ast__)}"
                "\n"
                "if __name__ == '__main__':\n"
                "    result = script()\n"
                "    print('[OK]', result)"
            )
    except ImportError:
        print("Warning: 'astor' library not found, skipping output generation.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Compile and execute a MiraScript file"
    )
    parser.add_argument(
        "-t",
        "--template",
        action="store_true",
        help="Indicates the input is a template file",
    )
    parser.add_argument(
        "-g",
        "--generate",
        action="store",
        help="Output generated code to the specified file",
    )
    parser.add_argument(
        "script_file",
        nargs="?",
        default="-",
        help="Path to the MiraScript file to compile (use '-' for stdin)",
    )
    args = parser.parse_args()
    if args.script_file == "-":
        script = sys.stdin.read()
        mode = "template" if args.template else "script"
    else:
        script_file = Path(args.script_file).resolve()
        if not script_file.is_file():
            print(f"Error: File '{script_file}' does not exist.")
            sys.exit(1)
        script = script_file.read_text(encoding="utf-8")
        mode = (
            "template" if args.template or script_file.suffix != ".mira" else "script"
        )

    # Compile and execute the script
    result, diagnostics = _compile(script, mode)

    if args.generate:
        _print_debug(result, args.generate)

    for diag in diagnostics:
        print(diag)

    if result is None:
        sys.exit(1)

    assert callable(result), "Compilation failed, result is not callable"

    try:
        print("[OK]", result())
    except Exception as e:
        traceback.print_exc()
