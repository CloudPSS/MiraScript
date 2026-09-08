import { compileSync, type VmContext, type VmValue } from '@mirascript/mirascript';
import { runInContext } from '@mirascript/mirascript/experimental-context';

export function execute(source: string, globals?: VmContext): Promise<VmValue> {
    return new Promise((resolve, reject) => runInContext(compileSync(source), globals, resolve, reject));
}
