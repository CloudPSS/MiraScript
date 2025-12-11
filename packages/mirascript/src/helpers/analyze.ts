import { generateBytecodeSync } from '../compiler/generate-bytecode.js';
import { DiagnosticCode, parseDiagnostics } from '../compiler/diagnostic.js';
import { REG_IDENTIFIER } from './constants.js';
import type { InputMode } from '../compiler/types.js';

/** 一个访问链，第一个元素为全局变量名称 */
export type GlobalReferenceChain = readonly [globalVariableName: string, ...fields: ReadonlyArray<string | number>];

const REG_SPILT = /\s*!?\s*\.\s*/u;
const REG_CHAIN = new RegExp(
    String.raw`^(${REG_IDENTIFIER.source})(?:${REG_SPILT.source}(?:\d+|${REG_IDENTIFIER.source}))*`,
    'u',
);

/**
 * 分析表达式依赖的全局变量
 */
export function analyzeGlobalReferences(expression: string, mode?: InputMode): GlobalReferenceChain[] {
    if (expression.trim() === '') {
        return [];
    }
    const [code, diagnostics] = generateBytecodeSync(expression, {
        input_mode: mode || 'Script',
        diagnostic_position_encoding: 'Utf16',
        diagnostic_error: false,
        diagnostic_warning: false,
        diagnostic_info: false,
        diagnostic_hint: false,
        diagnostic_reference: false,
        diagnostic_tag: true,
        trivia: false,
    });
    if (code == null || diagnostics.length === 0) {
        return [];
    }
    const parsedDiagnostics = parseDiagnostics(expression, diagnostics);
    const globals = parsedDiagnostics.tags.filter((t) => t.code === DiagnosticCode.GlobalVariable);
    if (globals.length === 0) {
        return [];
    }
    const lines = expression.split(/\r?\n/);
    const result = new Set<string>();
    for (const global of globals) {
        // 分析每个全局变量访问
        const line = lines[global.range.startLineNumber - 1];
        if (!line) continue;
        const access = line.slice(global.range.startColumn - 1);
        const chain = REG_CHAIN.exec(access);
        if (chain == null) continue;
        const globalName = chain[1];
        if (globalName?.length !== global.range.endColumn - global.range.startColumn) continue;
        const chainStr = chain[0].split(REG_SPILT).join('.');
        result.add(chainStr);
    }
    const accessChains: GlobalReferenceChain[] = [];
    for (const chainStr of result) {
        const parts = chainStr.split('.').map((part) => {
            const num = Number.parseInt(part, 10);
            return Number.isNaN(num) ? part : num;
        });
        accessChains.push(parts as unknown as GlobalReferenceChain);
    }
    return accessChains;
}
