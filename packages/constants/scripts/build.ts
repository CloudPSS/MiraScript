/// <reference types="node" />
import fs from 'node:fs/promises';
import * as lib from '../lib/constants.js';

const dts = await fs.readFile(new URL('../lib/constants.d.ts', import.meta.url), 'utf8');
const dtsLines = dts.split('\n');
// remove functions and empty lines
const filteredLines = dtsLines.filter((line) => {
    const trimmed = line.trim();
    return (
        !!trimmed &&
        !trimmed.startsWith('export function') &&
        !(trimmed.startsWith('/*') && trimmed.endsWith('*/') && trimmed.includes('disable'))
    );
});

let content = filteredLines.join('\n').replace('export class Config {', 'export declare class Config {');
const messages: Partial<Record<lib.DiagnosticCode, string>> = Object.create(null);
for (const code in lib.DiagnosticCode) {
    const num = Number(code);
    if (Number.isNaN(num)) continue;
    const message = lib.get_diagnostic_message(num as lib.DiagnosticCode);
    if (message && message !== 'Unknown error') {
        messages[num as lib.DiagnosticCode] = message;
    }
}

content += `
/** Diagnostic messages for MiraScript compiler and tools. */
export const DIAGNOSTIC_MESSAGES: Readonly<Partial<Record<DiagnosticCode, string>>> = Object.freeze(${JSON.stringify(messages, null, 2)} as const);
/** MiraScript keywords */
export const KEYWORDS = Object.freeze(${JSON.stringify(lib.keywords())} as const);
/** MiraScript constant keywords */
export const CONSTANT_KEYWORDS = Object.freeze(${JSON.stringify(lib.constant_keywords())} as const);
/** MiraScript control flow keywords */
export const CONTROL_KEYWORDS = Object.freeze(${JSON.stringify(lib.control_keywords())} as const);
/** MiraScript numeric keywords */
export const NUMERIC_KEYWORDS = Object.freeze(${JSON.stringify(lib.numeric_keywords())} as const);
/** MiraScript reserved keywords */
export const RESERVED_KEYWORDS = Object.freeze(${JSON.stringify(lib.reserved_keywords())} as const);
`;
await fs.mkdir(new URL('../src/', import.meta.url), { recursive: true });
await fs.writeFile(new URL('../src/constants.g.ts', import.meta.url), content, 'utf8');
