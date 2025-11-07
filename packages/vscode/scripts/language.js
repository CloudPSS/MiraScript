import { load } from 'js-yaml';
import fs from 'node:fs/promises';

await fs.mkdir(new URL('../syntaxes', import.meta.url), { recursive: true });

const shared = load(await fs.readFile(new URL('./shared.tmLanguage.yaml', import.meta.url), 'utf-8'));

for (const file of await fs.readdir(new URL('./', import.meta.url), { withFileTypes: true })) {
    if (!file.name.endsWith('.yaml')) continue;
    if (file.name === 'shared.tmLanguage.yaml') continue;

    const content = await fs.readFile(new URL(file.name, import.meta.url), 'utf-8');
    let data = load(content);
    if (!data.repository) {
        data = { ...shared, ...data };
    }
    await fs.writeFile(
        new URL(`../syntaxes/${file.name.replace(/\.yaml$/, '.json')}`, import.meta.url),
        JSON.stringify(data, null, 2),
        'utf-8',
    );
}
