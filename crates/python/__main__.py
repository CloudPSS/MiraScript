import mirascript

if __name__ == "__main__":
    # Example usage of the mirascript module
    # This will compile a simple Python script and print the result
    result = mirascript.compile("print('Hello, world!')", input_mode="script")
    print(result)
