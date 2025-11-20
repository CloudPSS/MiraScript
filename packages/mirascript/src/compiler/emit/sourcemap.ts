import type { IRange } from '../diagnostic.js';
import type { ScriptInput, TranspileOptions } from '../types.js';
import { SourceMapGenerator } from 'source-map-js';
import { GLOBAL_HINT, SCRIPT_PREFIX } from './constants.js';
import type { GlobalMap } from './globals.js';

const ORIGIN = `mira://MiraScript/`;
const PREFIX = '//# ';
const SOURCE_URL = `${PREFIX}sourceURL`;
const SOURCE_MAPPING_URL = `${PREFIX}sourceMappingURL`;

const toDataUrl: (json: string) => string =
    typeof Buffer == 'function' && typeof Buffer.from == 'function'
        ? (s) => `data:application/json;base64,${Buffer.from(s, 'utf8').toString('base64')}`
        : (s) => `data:application/json;charset=utf-8,${encodeURIComponent(s)}`;

/** 添加全局变量的源映射 */
function addGlobalMappings(globalLine: string, fileName: string, map: SourceMapGenerator, globals: GlobalMap) {
    let globalFile = `global;\n`;
    map.addMapping({
        generated: {
            line: 3,
            column: globalLine.indexOf(`global = `),
        },
        original: {
            line: 1,
            column: 0,
        },
        source: fileName,
        name: 'global',
    });
    map.addMapping({
        generated: {
            line: 3,
            column: SCRIPT_PREFIX.length,
        },
        original: {
            line: 1,
            column: 7,
        },
        source: fileName,
        name: '',
    });
    let i = 1;
    let pos = globalLine.indexOf(GLOBAL_HINT, SCRIPT_PREFIX.length) + GLOBAL_HINT.length;
    for (const p of globals.values()) {
        i++;
        if (pos < 0) break;
        const { v, n } = p;
        pos = globalLine.indexOf(v, pos);
        if (pos < 0) break;
        map.addMapping({
            generated: {
                line: 3,
                column: pos,
            },
            original: {
                line: i,
                column: 0,
            },
            source: fileName,
            name: n,
        });
        globalFile += `${n};\n`;
    }
    map.addMapping({
        generated: {
            line: 3,
            column: pos,
        },
        original: {
            line: i,
            column: 0,
        },
        source: fileName,
        name: '',
    });
    map.setSourceContent(fileName, globalFile);
}

let sourceId = 1;
/** 创建源映射 */
export function createSourceMap(
    source: ScriptInput | undefined,
    sourcemaps: readonly IRange[],
    globalLine: string | undefined,
    globals: GlobalMap,
    options: TranspileOptions,
): [string, string] {
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
                // 前两行固定为：
                // (function anonymous($Add,$Aeq, ...
                // ) {
                line: i + 3,
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
                line: 3,
                column: SCRIPT_PREFIX.length - 'CpEnter();'.length,
            },
            original: {
                line: 1,
                column: 0,
            },
            source: fileName,
        });
    }
    if (globalLine?.includes(GLOBAL_HINT)) {
        addGlobalMappings(globalLine, `${fileName} <globals>`, map, globals);
    }
    const sourceURL = hasSchema ? fileName : `${ORIGIN}${fileName}`;
    const dataUrl = toDataUrl(map.toString());
    return [
        // Prevent source map from being recognized as of this file
        `${SOURCE_URL}=${sourceURL}.js`,
        `${SOURCE_MAPPING_URL}=${dataUrl}`,
    ];
}
