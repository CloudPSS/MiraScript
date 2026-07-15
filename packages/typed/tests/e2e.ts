import test from 'ava';
import { parse, simplify, stringify } from '../dist/index.js';
import { lib } from '@mirascript/mirascript/subtle';
import type { VmLibOption } from '../../mirascript/src/vm/lib/helpers.ts';

const lt = test.macro<[name: string, param: string, value: VmLibOption]>({
    exec: (t, name, param, value) => {
        const typeStr = param === 'returns' ? (value.returns?.type ?? 'nil') : (value.params?.[param]?.type ?? 'nil');
        const parsed = parse(typeStr);
        const simplified = simplify(parsed);
        const stringified = stringify(simplified);
        const reparsed = parse(stringified);
        t.deepEqual(reparsed, simplified);
    },
    title: (t, name, param, value) => {
        let type: string;
        if (param === 'returns') {
            type = value.returns?.type ?? 'nil';
        } else {
            type = value.params?.[param]?.type ?? 'nil';
        }
        return `parse lib ${name} > ${param} (${type})`;
    },
});

type Lib = {
    [name: string]: VmLibOption | Lib;
};
function testLib(lib: Lib) {
    for (const [name, value] of Object.entries(lib)) {
        if ('summary' in value || 'params' in value || 'returns' in value) {
            for (const param of Object.keys(value.params ?? {})) {
                test(lt, name, param, value as VmLibOption);
            }
            if (value.returns) {
                test(lt, name, 'returns', value as VmLibOption);
            }
        } else {
            testLib(value as Lib);
        }
    }
}
testLib(lib);
