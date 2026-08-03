from __future__ import annotations
from argparse import ArgumentParser


def create_parser() -> ArgumentParser:
    parser = ArgumentParser(description="Compile and execute a MiraScript file")

    parser.add_argument(
        "-t",
        "--template",
        action="store_true",
        help="Indicates the input is a template file",
    )
    parser.add_argument(
        "-g",
        "--generate",
        metavar="output.py",
        action="store",
        help="Output generated code to the specified file",
    )
    parser.add_argument(
        "-e",
        "--eval",
        action="store",
        metavar="SCRIPT",
        help="Evaluate a MiraScript code snippet directly from the command line",
    )
    parser.add_argument(
        "script_file",
        nargs="?",
        help="Path to the MiraScript file to compile (use '-' for stdin)",
    )
    parser.add_argument(
        "-v",
        "--variable",
        action="append",
        metavar="NAME=VALUE",
        help="Define a variable for evaluation (can be used multiple times)",
    )
    return parser
