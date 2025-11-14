from mirascript.vm.operations import *
from mirascript.vm.types.checker import is_vm_const
from mirascript.vm.types.extern import VmExtern
from mirascript.vm.helpers import GlobalFallback,Element,ArrayRange,ArrayRangeExclusive,ElementOpt,Upvalue
from mirascript.vm.helpers import CpEnter,CpExit
Uninitialized = type("Uninitialized", (), {})()
def script(context=GlobalFallback(), *args, **kwargs):
    try:
        CpEnter()
        (_, var_1_1, var_1_2, var_1_3, var_1_4, var_1_5, var_1_6, var_1_7, var_1_8) = (Uninitialized, Uninitialized, Uninitialized, Uninitialized, Uninitialized, Uninitialized, Uninitialized, Uninitialized, Uninitialized)
        var_1_5 = GetGlobal_(context, 'matrix')
        var_1_5 = Get_(var_1_5, 'identity')
        if var_1_5 is not None:
            var_1_6 = 1001.0
            var_1_4 = Call_(var_1_5, *(var_1_6,))
        else:
            var_1_4 = None

        def var_1_7(var_2_1=None, *args, **kwargs):
            try:
                CpEnter()
                (_, var_2_2) = (Uninitialized, Uninitialized)
                var_2_2 = Call_(context['sum'], *(var_2_1,))
                return var_2_2
            finally:
                CpExit()
        var_1_3 = Call_(context['map'], *(var_1_4, var_1_7))
        var_1_2 = Call_(context['sum'], *(var_1_3,))
        var_1_8 = 1001.0
        _ = Call_(context['t_eq'], *(var_1_2, var_1_8))
        var_1_1 = None
        return var_1_1
    finally:
        CpExit()
script()