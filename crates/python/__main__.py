from mirascript import compile, Config

if __name__ == "__main__":
    # Example usage of the mirascript module
    # This will compile a simple Python script and print the result
    result = compile("'Hello, world!'", Config(input_mode="script"))
    assert callable(result), "Compilation failed, result is not callable"
    print(result())  # Should print: Hello, world!
