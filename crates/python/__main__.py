import mirascript
import traceback
import os
if __name__ == "__main__":
    # Example usage of the mirascript module
    # This will compile a simple Python script and print the result
    # script_file='01_hello_world.mira'
    # script_file='02_comments_identifiers.mira'
    # script_file='03_literals.mira'
    # script_file='06_variables.mira'
    # script_file='11_interpolation.mira'
    # script_file='15_array.mira'
    # script_file='20_operators.mira'
    # script_file='21_extension_calls.mira'
    script_file='25_control_flow.mira'
    script_file='30_pattern.mira'
    script_file='31_advanced_patterns.mira'
    script_file='40_functions.mira'
    script_file='41_fib.mira'
    script_file='50_null_safety.mira'
    script_file='55_error_handling.mira'
    script_file='60_math.mira'
    script_file='70_algorithms.mira'
    
    
    fs= open(os.path.join(os.path.dirname(__file__),'..','..','examples',script_file),'r',encoding='utf-8')
    # print(fs.read())
    script=fs.read()
    fs.close()
    try:
        result, diagnostics = mirascript.compile(script, mirascript.Config(input_mode="script"))
        assert callable(result), "Compilation failed, result is not callable"
        print("result",result())  # Should print: Hello, world!
    except Exception as e:
        traceback.print_exc()
        print(f"Error during compilation or execution: {e}")
    