import { load } from 'js-yaml';
import fs from 'node:fs/promises';

const output = new URL('../syntaxes/', import.meta.url);
await fs.mkdir(output, { recursive: true });

for (const filename of [
    'mirascript.tmLanguage.json',
    'mirascript-template.tmLanguage.json',
    'mirascript-doc.tmLanguage.json',
]) {
    const source = new URL(import.meta.resolve(`@mirascript/textmate/${filename}`));
    await fs.copyFile(source, new URL(filename, output));
}

for (const file of await fs.readdir(new URL('./', import.meta.url), { withFileTypes: true })) {
    if (!file.name.endsWith('.yaml')) continue;

    const content = await fs.readFile(new URL(file.name, import.meta.url), 'utf8');
    const data = load(content);
    await fs.writeFile(new URL(file.name.replace(/\.yaml$/, '.json'), output), JSON.stringify(data, null, 2), 'utf8');
}
