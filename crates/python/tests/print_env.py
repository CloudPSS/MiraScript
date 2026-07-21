from mirascript._vm.types.context import get_shared_context

if __name__ == "__main__":
    print("VmSharedContext keys:", list(get_shared_context().keys()))
