import { createHighlighterCore } from '@shikijs/core';
import { createOnigurumaEngine } from '@shikijs/engine-oniguruma';
import wasm from '@shikijs/engine-oniguruma/wasm-inlined';
import fs from 'node:fs/promises';
import { grammars } from '../src/index.ts';

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
