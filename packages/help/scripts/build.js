import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';

const packageRoot = path.resolve(import.meta.dirname, '..');
const srcRoot = path.join(packageRoot, 'src');
const distRoot = path.join(packageRoot, 'dist');

/**
 * NOTE:
 * - Keys are the actual keyword/operator tokens used in source code.
 * - Values are the raw Markdown texts.
 * - Filenames are platform-safe (avoid `:` `?` etc on Windows).
 */

const KEYWORD_SOURCES = [
    ['true', 'keyword/true.md'],
    ['false', 'keyword/false.md'],
    ['nil', 'keyword/nil.md'],
    ['nan', 'keyword/nan.md'],
    ['inf', 'keyword/inf.md'],

    ['_', 'keyword/underscore.md'],
    ['global', 'keyword/global.md'],

    ['in', 'keyword/in.md'],
    ['is', 'keyword/is.md'],
    ['and', 'keyword/and.md'],
    ['or', 'keyword/or.md'],
    ['not', 'keyword/not.md'],

    ['type', 'keyword/type.md'],

    ['if', 'keyword/if.md'],
    ['else', 'keyword/else.md'],
    ['match', 'keyword/match.md'],
    ['case', 'keyword/case.md'],
    ['for', 'keyword/for.md'],
    ['while', 'keyword/while.md'],
    ['loop', 'keyword/loop.md'],
    ['break', 'keyword/break.md'],
    ['continue', 'keyword/continue.md'],
    ['return', 'keyword/return.md'],

    ['fn', 'keyword/fn.md'],
    ['op', 'keyword/op.md'],
    ['let', 'keyword/let.md'],
    ['const', 'keyword/const.md'],
    ['mut', 'keyword/mut.md'],
    ['where', 'keyword/where.md'],

    ['mod', 'keyword/mod.md'],
    ['pub', 'keyword/pub.md'],
    ['use', 'keyword/use.md'],

    ['effect', 'keyword/effect.md'],
    ['try', 'keyword/try.md'],
    ['handle', 'keyword/handle.md'],
    ['finally', 'keyword/finally.md'],
    ['perform', 'keyword/perform.md'],
    ['resume', 'keyword/resume.md'],
];

const OPERATOR_SOURCES = [
    ['(', 'operator/open_paren.md'],
    [')', 'operator/close_paren.md'],
    ['[', 'operator/open_bracket.md'],
    [']', 'operator/close_bracket.md'],
    [':', 'operator/colon.md'],
    ['?', 'operator/question.md'],
    ['?:', 'operator/question_colon.md'],
    ['::', 'operator/colon_colon.md'],
    [',', 'operator/comma.md'],
    ['.', 'operator/dot.md'],
    ['->', 'operator/arrow.md'],

    ['..', 'operator/spread_range.md'],
    ['..<', 'operator/half_open_range.md'],

    ['+', 'operator/plus.md'],
    ['+=', 'operator/plus_assign.md'],
    ['-', 'operator/minus.md'],
    ['-=', 'operator/minus_assign.md'],
    ['*', 'operator/asterisk.md'],
    ['*=', 'operator/asterisk_assign.md'],
    ['/', 'operator/slash.md'],
    ['/=', 'operator/slash_assign.md'],
    ['%', 'operator/percent.md'],
    ['%=', 'operator/percent_assign.md'],
    ['^', 'operator/caret.md'],
    ['^=', 'operator/caret_assign.md'],

    ['!', 'operator/exclamation.md'],
    ['&&', 'operator/logical_and.md'],
    ['&&=', 'operator/logical_and_assign.md'],
    ['||', 'operator/logical_or.md'],
    ['||=', 'operator/logical_or_assign.md'],
    ['??', 'operator/null_coalescing.md'],
    ['??=', 'operator/null_coalescing_assign.md'],

    ['=', 'operator/assign.md'],
    ['==', 'operator/equal.md'],
    ['!=', 'operator/not_equal.md'],
    ['=~', 'operator/tilde_equal.md'],
    ['!~', 'operator/tilde_not_equal.md'],
    ['>', 'operator/greater.md'],
    ['>=', 'operator/greater_equal.md'],
    ['<', 'operator/less.md'],
    ['<=', 'operator/less_equal.md'],

    [';', 'operator/semicolon.md'],
    ['{', 'operator/open_brace.md'],
    ['}', 'operator/close_brace.md'],
];

/**
 * Read a markdown file under `src/`.
 * @param {string} relativePath
 * @returns {Promise<string>}
 */
async function readMarkdown(relativePath) {
    const fullPath = path.join(srcRoot, relativePath);
    return readFile(fullPath, 'utf8');
}

/**
 * Load markdown docs and return `[token, rawMarkdown]` entries.
 * @param {Array<[string, string]>} pairs
 * @returns {Promise<Array<[string, string]>>}
 */
async function loadDocs(pairs) {
    /** @type {Array<[string, string]>} */
    const entries = [];
    for (const [token, mdPath] of pairs) {
        const content = await readMarkdown(mdPath);
        entries.push([token, content]);
    }
    return entries;
}

/**
 * Render an object literal with string keys and raw markdown values.
 * @param {Array<[string, string]>} entries
 * @returns {string}
 */
function renderObjectLiteral(entries) {
    const lines = entries.map(([k, v]) => `  ${JSON.stringify(k)}: ${JSON.stringify(v)},`);
    return `({\n${lines.join('\n')}\n})`;
}

/**
 * Render a `.d.ts` object type with explicit string-literal keys.
 * @param {Array<[string, string]>} entries
 * @returns {string}
 */
function renderDtsObjectType(entries) {
    const lines = entries.map(([k]) => `  ${JSON.stringify(k)}: string;`);
    return `{\n${lines.join('\n')}\n}`;
}

/**
 * Build `dist/index.js` and `dist/index.d.ts`.
 * @returns {Promise<void>}
 */
async function main() {
    const keywordEntries = await loadDocs(KEYWORD_SOURCES);
    const operatorEntries = await loadDocs(OPERATOR_SOURCES);

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
