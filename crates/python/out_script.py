from mirascript.vm.operations import *
from mirascript.vm.types.checker import is_vm_const
from mirascript.vm.types.extern import VmExtern
from mirascript.vm.helpers import GlobalFallback,Element,ArrayRange,ArrayRangeExclusive,ElementOpt,Upvalue
from mirascript.vm.helpers import CpEnter,CpExit
Uninitialized = type("Uninitialized", (), {})()
def script(context=GlobalFallback(), *args, **kwargs):
    try:
        CpEnter()
        (_, var_1_1, var_1_2, var_1_3) = (Uninitialized, Uninitialized, Uninitialized, Uninitialized)
        var_1_2 = True
        if ToBoolean_(var_1_2) != False:

            def var_1_3(*args, **kwargs):
                try:
                    CpEnter()
                    (_, var_2_1, var_2_2, var_2_3) = (Uninitialized, Uninitialized, Uninitialized, Uninitialized)
                    var_2_3 = GetGlobal_(context, 'v_extern')
                    var_2_1 = var_2_3
                    print('var_2_1:',var_2_1,type(var_2_1))  # --- DEBUG ---
                    _ = Call_(var_2_1)
                    var_2_2 = None
                    return var_2_2
                finally:
                    CpExit()
            # _ = Call_(context['t_throws'], *(var_1_3,))
            var_1_3()
            var_1_1 = None
        else:
            var_1_1 = None
        return var_1_1
    finally:
        CpExit()
script()