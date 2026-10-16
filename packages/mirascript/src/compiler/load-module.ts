import { loadModule } from '@mirascript/bindings';

try {
    await loadModule();
} catch {
    // Ignore, will be rethrown when the module is actually used.
}
