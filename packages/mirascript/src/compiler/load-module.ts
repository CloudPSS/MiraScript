import { loadModule } from '@mirascript/bindings';

try {
    await loadModule();
} catch (error) {
    // Ignore, will be rethrown when the module is actually used.
}
