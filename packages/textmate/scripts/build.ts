import { createHighlighterCore } from '@shikijs/core';
import { createOnigurumaEngine } from '@shikijs/engine-oniguruma';
import wasm from '@shikijs/engine-oniguruma/wasm-inlined';
import fs from 'node:fs/promises';
import { grammars } from './index.ts';

const dist = new URL('../dist/', import.meta.url);
await fs.mkdir(dist, { recursive: true });

const highlighter = await createHighlighterCore({
    langs: grammars,
    themes: [],
    engine: createOnigurumaEngine(wasm),
});

try {
    for (const grammar of grammars) {
        highlighter.getLanguage(grammar.name).tokenizeLine('let value = "$(1 + 2)";', null);
    }
} finally {
    highlighter.dispose();
}

for (const grammar of grammars) {
    const filename = `${grammar.name}.tmLanguage.json`;
    await fs.writeFile(
        new URL(filename, dist),
        `${JSON.stringify(
            {
                $schema: 'https://raw.githubusercontent.com/martinring/tmlanguage/master/tmlanguage.json',
                ...grammar,
            },
            null,
            2,
        )}\n`,
        'utf8',
    );
}

const js = [];
const dts = ['import type { LanguageRegistration } from "@shikijs/types";'];
for (const grammar of grammars) {
    const { name } = grammar;
    const id = name.replaceAll(/-[a-z]/g, (match) => match[1].toUpperCase());
    js.push(`import ${id} from './${grammar.name}.tmLanguage.json' with { type: 'json' };`, `export { ${id} };`);
    dts.push(`export declare const ${id}: LanguageRegistration;`);
    if (id !== name) {
        js.push(`export { ${id} as '${name}' };`);
        dts.push(`export { ${id} as '${name}' };`);
    }
}

await fs.writeFile(new URL('index.js', dist), js.join('\n') + '\n', 'utf8');

await fs.writeFile(new URL('index.d.ts', dist), dts.join('\n') + '\n', 'utf8');
