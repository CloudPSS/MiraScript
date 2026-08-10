import { createHighlighterCore } from '@shikijs/core';
import { createOnigurumaEngine } from '@shikijs/engine-oniguruma';
import wasm from '@shikijs/engine-oniguruma/wasm-inlined';
import fs from 'node:fs/promises';
import { grammars } from './grammar.js';

const dist = new URL('../dist/', import.meta.url);
const syntaxes = new URL('./syntaxes/', dist);
await fs.mkdir(syntaxes, { recursive: true });

const highlighter = await createHighlighterCore({
    langs: grammars,
    themes: [],
    engine: createOnigurumaEngine(wasm),
});

try {
    for (const grammar of grammars) {
        highlighter.getLanguage(grammar.name).tokenizeLine('let value = "$(1 + 2)";');
    }
} finally {
    highlighter.dispose();
}

const filenames = new Map([
    ['mirascript', 'mira.tmLanguage.json'],
    ['mirascript-template', 'miratpl.tmLanguage.json'],
    ['mirascript-doc', 'mira-doc.tmLanguage.json'],
]);

for (const grammar of grammars) {
    const filename = filenames.get(grammar.name);
    await fs.writeFile(new URL(filename, syntaxes), `${JSON.stringify(grammar, null, 2)}\n`, 'utf8');
    await fs.writeFile(
        new URL(`${grammar.name}.js`, dist),
        `export default ${JSON.stringify(grammar, null, 2)};\n`,
        'utf8',
    );
}

await fs.writeFile(
    new URL('index.js', dist),
    [
        "export { default as mirascript } from './mirascript.js';",
        "export { default as mirascriptTemplate } from './mirascript-template.js';",
        "export { default as mirascriptDoc } from './mirascript-doc.js';",
        '',
    ].join('\n'),
    'utf8',
);

const declaration = `import type { LanguageRegistration } from '@shikijs/types';
export declare const mirascript: LanguageRegistration;
export declare const mirascriptTemplate: LanguageRegistration;
export declare const mirascriptDoc: LanguageRegistration;
`;
await fs.writeFile(new URL('index.d.ts', dist), declaration, 'utf8');
