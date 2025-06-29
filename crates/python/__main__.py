import mirascript

if __name__ == "__main__":
    # Example usage of the mirascript module
    # This will compile a simple Python script and print the result
    result, diagnostics = mirascript.compile(
        "('Hello, world!')", mirascript.Config(input_mode="script")
    )
    assert callable(result), "Compilation failed, result is not callable"
    print(diagnostics, result())  # Should print: Hello, world!
