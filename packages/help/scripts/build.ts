import { mkdir, readFile, readdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { load } from 'js-yaml';

const packageRoot = path.resolve(import.meta.dirname, '..');
const srcRoot = path.join(packageRoot, '../../docs/references');
const distRoot = path.join(packageRoot, 'dist');

/**
 * Front-matter driven docs.
 *
 * Each markdown file under `src/keyword` and `src/operator` must have:
 *
 * ---
 * token: "..."   # actual token used in source code
 * ---
 */

/** Read a markdown file under `src/`. */
async function readMarkdown(relativePath: string): Promise<string> {
    const fullPath = path.join(srcRoot, relativePath);
    return await readFile(fullPath, 'utf8');
}

const FRONT_MATTER_RE = /^---\r?\n([\s\S]*?)\r?\n---\r?\n/;

/** Front-matter attributes of a markdown doc. */
interface FrontMatter {
    /** The token(s) mapped to the doc body. */
    token?: unknown;
    /** Fallback token when `token` is missing. */
    title?: unknown;
    /** Whether the token is reserved and must be skipped. */
    reserved?: unknown;
}

/** A doc item before the token is normalized to a string. */
interface RawDocItem {
    /** The token, possibly an array of tokens. */
    token: string | string[];
    /** The markdown body. */
    body: string;
    /** Whether the token is reserved. */
    reserved: boolean;
    /** The source file relative path. */
    file: string;
}

/** A doc item with a normalized string token. */
interface DocItem {
    /** The token. */
    token: string;
    /** The markdown body. */
    body: string;
    /** Whether the token is reserved. */
    reserved: boolean;
    /** The source file relative path. */
    file: string;
}

/** Extract front-matter and strip it from markdown. */
function splitFrontMatter(markdown: string, relativePath: string): { attributes: FrontMatter; body: string } {
    const m = FRONT_MATTER_RE.exec(markdown);
    if (!m) {
        throw new Error(`Missing front-matter in ${relativePath}. Add token/order mapping.`);
    }

    // The regex guarantees both groups exist when `m` is truthy.
    const yaml = m[1];
    const fullMatch = m[0];
    if (yaml === undefined || fullMatch === undefined) {
        throw new Error(`Missing front-matter in ${relativePath}. Add token/order mapping.`);
    }
    const attributes = load(yaml) as FrontMatter;
    let body = markdown.slice(fullMatch.length);
    // Trim leading newlines and tailing newlines
    body = body.replace(/^\r?\n+/, '').replace(/\r?\n+$/, '') + '\n';
    return { attributes, body };
}

/** Load docs under a folder like `keyword` or `operator`. */
async function loadDocsFromFolder(folder: string): Promise<Array<[string, string]>> {
    const dirPath = path.join(srcRoot, folder);
    const dirents = await readdir(dirPath, { withFileTypes: true });

    const items: DocItem[] = [];

    /** Check and add an item. */
    function putItem(item: RawDocItem): void {
        if (Array.isArray(item.token)) {
            for (const token of item.token) {
                items.push({ token, body: item.body, reserved: item.reserved, file: item.file });
            }
            return;
        }
        if (!item.token.length) {
            throw new TypeError(`Invalid front-matter field 'token' in ${item.file}`);
        }
        if (item.reserved) {
            return; // skip reserved tokens
        }
        items.push({ token: item.token, body: item.body, reserved: item.reserved, file: item.file });
    }

    for (const dirent of dirents) {
        if (!dirent.isFile()) continue;
        if (!dirent.name.endsWith('.md')) continue;

        const relativePath = path.posix.join(folder, dirent.name);
        const markdown = await readMarkdown(relativePath);
        const { attributes, body } = splitFrontMatter(markdown, relativePath);
        const token = attributes.token ?? attributes.title;
        if (typeof token !== 'string' && !Array.isArray(token)) {
            throw new TypeError(`Invalid front-matter field 'token' in ${relativePath}`);
        }
        putItem({ token, body, reserved: Boolean(attributes.reserved), file: relativePath });
    }

    const seenToken = new Set<string>();
    const entries: Array<[string, string]> = [];
    for (const item of items) {
        if (seenToken.has(item.token)) {
            throw new Error(`Duplicate token '${item.token}' (found in ${item.file})`);
        }
        seenToken.add(item.token);
        entries.push([item.token, item.body]);
    }

    return entries;
}

/** Render an object literal with string keys and raw markdown values. */
function renderObjectLiteral(entries: Array<[string, string]>): string {
    const lines = entries.map(([k, v]) => `  ${JSON.stringify(k)}: ${JSON.stringify(v)},`);
    lines.unshift('  __proto__: null,');
    return `Object.freeze({\n${lines.join('\n')}\n})`;
}

/** Render a `.d.ts` object type with explicit string-literal keys. */
function renderDtsObjectType(entries: Array<[string, string]>): string {
    const lines = entries.map(([k]) => `  readonly ${JSON.stringify(k)}: string;`);
    return `{\n${lines.join('\n')}\n}`;
}

/** Build `dist/index.js` and `dist/index.d.ts`. */
async function main(): Promise<void> {
    const keywordEntries = await loadDocsFromFolder('keyword');
    const operatorEntries = await loadDocsFromFolder('operator');

    await mkdir(distRoot, { recursive: true });

    const js = [
        '/* Generated by scripts/build.js. Do not edit manually. */',
        '',
        `export const KEYWORDS = ${renderObjectLiteral(keywordEntries)};`,
        `export const OPERATORS = ${renderObjectLiteral(operatorEntries)};`,
        '',
    ].join('\n');

    const dts = [
        '/* Generated by scripts/build.js. Do not edit manually. */',
        '',
        `export declare const KEYWORDS: ${renderDtsObjectType(keywordEntries)};`,
        `export declare const OPERATORS: ${renderDtsObjectType(operatorEntries)};`,
        '',
        'export type Keyword = keyof typeof KEYWORDS;',
        'export type Operator = keyof typeof OPERATORS;',
        '',
    ].join('\n');

    await writeFile(path.join(distRoot, 'index.js'), js, 'utf8');
    await writeFile(path.join(distRoot, 'index.d.ts'), dts, 'utf8');
}

main().catch((err) => {
    // eslint-disable-next-line no-console
    console.error(err);
    process.exitCode = 1;
});
