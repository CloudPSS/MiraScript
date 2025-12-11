import type { IRange } from '../diagnostic.js';
import type { ScriptInput, TranspileOptions } from '../types.js';
import { SourceMapGenerator } from 'source-map-js';
import { SCRIPT_PREFIX } from './constants.js';
import { toJsLiteral } from './consts.js';

const ORIGIN = `mira://MiraScript/`;
const { SOURCE_URL, SOURCE_MAPPING_URL } = ((source, mapping, url) => {
    // 避免被识别为当前文件的源映射
    const prefix = '//# ';
    return {
        SOURCE_URL: prefix.concat(source, url),
        SOURCE_MAPPING_URL: prefix.concat(source, mapping, url),
    };
})(`source`, `Mapping`, `URL`);
// 前 3 行固定为：
// (function anonymous($Add,$Aeq, ...
// ) {
// 'use strict';
const SOURCE_OFFSET = 4;

/**
 * Node.js Buffer 类型的简易声明，@mirascript/playground 调试环境下会直接加载此文件
 */
declare class Buffer {
    /** @inheritdoc */
    static from(str: string, encoding: 'utf8'): Buffer;
    /** @inheritdoc */
    toString(encoding: 'base64'): string;
}

const toDataUrl: (json: string) => string =
    typeof Buffer == 'function' && typeof Buffer.from == 'function'
        ? (s) => `data:application/json;base64,${Buffer.from(s, 'utf8').toString('base64')}`
        : (s) => `data:application/json;charset=utf-8,${encodeURIComponent(s)}`;

let sourceId = 1;
/** 创建源映射 */
export function createSourceMap(
    source: ScriptInput | undefined,
    sourcemaps: readonly IRange[],
    codeLines: string[],
    functions: readonly number[],
    options: TranspileOptions,
): void {
    let fileName = (options.fileName ?? '').trim();
    const hasSchema = /^\w+:/.test(fileName);
    if (!hasSchema) {
        if (fileName.startsWith('/')) {
            fileName = fileName.replace(/^\/+\s*/, '');
        }
        if (!fileName) {
            fileName = `${sourceId++}.${options.input_mode === 'Template' ? 'miratpl' : 'mira'}`;
        }
    }
    const map = new SourceMapGenerator({
        file: fileName + '.js',
    });
    if (typeof source === 'string') {
        map.setSourceContent(fileName, source);
    }
    let hasStartMap = false;
    for (let i = 1; i < sourcemaps.length; i++) {
        const range = sourcemaps[i];
        if (!range) break;
        if (!hasStartMap && range.startLineNumber === 1 && range.startColumn === 1) {
            hasStartMap = true;
        }
        map.addMapping({
            generated: {
                line: i + SOURCE_OFFSET,
                column: 0,
            },
            original: {
                line: range.startLineNumber,
                column: range.startColumn - 1,
            },
            source: fileName,
        });
    }
    if (!hasStartMap) {
        map.addMapping({
            generated: {
                line: SOURCE_OFFSET,
                column: SCRIPT_PREFIX.length - 'CpEnter();'.length,
            },
            original: {
                line: 1,
                column: 0,
            },
            source: fileName,
        });
    }
    if (typeof source === 'string') {
        const lines = source.split(/\r?\n/);
        for (const codeLineNo of functions) {
            const mapping = sourcemaps[codeLineNo];
            const sourceLine = mapping ? lines[mapping.startLineNumber - 1] : undefined;
            const fnName =
                mapping && sourceLine ? sourceLine.slice(mapping.startColumn - 1, mapping.endColumn - 1) : '';
            if (!fnName) continue;
            const codeLine = codeLines[codeLineNo]!;
            codeLines[codeLineNo] = codeLine.replace(`= Fn(null,`, `= Fn(${toJsLiteral(fnName)},`);
        }
    }
    const sourceURL = hasSchema ? fileName : `${ORIGIN}${fileName}`;
    const dataUrl = toDataUrl(map.toString());
    codeLines.push(
        // Prevent source map from being recognized as of this file
        `${SOURCE_URL}=${sourceURL}.js`,
        `${SOURCE_MAPPING_URL}=${dataUrl}`,
    );
}
