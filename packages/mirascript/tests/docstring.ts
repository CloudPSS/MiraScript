import test from 'ava';
import { compile } from '@mirascript/mirascript';
import { lib } from '@mirascript/mirascript/subtle';
import type { VmLibOption } from '../src/vm/lib/helpers.ts';

function extractExpectedOutput(example: string): string | null {
    if (!example.includes('//')) return null;

    const splitted = example.split('//');
    if (splitted.length === 2) {
        // Single-line example without newline, return the part after '//'
        return splitted[1]!.trim();
    }

    // Multi-line example, extract the final lines prefixed with '//'
    const lines = example.split('\n');
    let result = '';
    for (let i = lines.length - 1; i >= 0; i--) {
        const line = lines[i]!.trim();
        if (!line.startsWith('//')) break;
        result = line.slice(2).trim() + '\n' + result;
    }
    return result || null;
}

const docExamples = test.macro<[string, VmLibOption]>({
    exec: async (t, name, data) => {
        if (!data.examples?.length) {
            t.pass('No examples to test');
            return;
        }
        for (const example of data.examples) {
            const expectedStr = extractExpectedOutput(example);
            if (!expectedStr) {
                if (example.endsWith(';')) {
                    // Not pure function, just test compilation
                    await compile(example);
                    t.pass('Compiled successfully');
                    continue;
                }
                return t.fail('Expected output not found in example');
            }
            const expected = (await compile(expectedStr))();
            const result = (await compile(example))();
            t.deepEqual(result, expected);
        }
    },
    title: (providedTitle = 'doc example of', name) => `${providedTitle} ${name}`,
});

type Lib = {
    [key: string]: VmLibOption | Lib;
};
function testLib(root: string | null, lib: Lib) {
    for (const key in lib) {
        const item = lib[key];
        if (item == null) continue;
        const fullKey = root ? `${root}.${key}` : key;
        if ('summary' in item && typeof item.summary == 'string') {
            test(docExamples, fullKey, item);
        } else {
            testLib(fullKey, item as Lib);
        }
    }
}

testLib(null, lib);
