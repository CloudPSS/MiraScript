from .constants import kVmScript, kVmContext, kVmFunction


def isVmScript(value):
    return callable(value) and getattr(value, kVmScript, False)


def isVmContext(context):
    return context is not None and getattr(context, kVmContext, False)


def isVmFunction(value):
    return callable(value) and getattr(value, kVmFunction, False)
