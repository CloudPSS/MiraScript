import { toString } from '../../helpers/convert/to-string.js';
import { toJsLiteral } from './consts.js';
import type { Emitter } from './index.js';

/** 全局变量描述符 */
export type GlobalDesc = {
    /** 变量编译后的名称 */
    v: string;
    /** 变量名字的字面量，是一个 JS 字符串字面量 */
    l: string;
    /** 变量的表达式，用于获取全局变量 */
    e: string;
    /** 变量的名称 */
    n: string;
};

/** 全局变量映射，key 为变量名在常量表中的索引 */
export type GlobalMap = Map<number, GlobalDesc>;

/** 获取全局变量映射 */
export function readGlobal(emitter: Emitter, constIdx: number): GlobalDesc {
    const { globals } = emitter;
    const cached = globals.get(constIdx);
    if (cached) return cached;
    const constName = emitter.constVals[constIdx]!;
    let name, lit;
    if (typeof constName == 'string') {
        name = constName;
        lit = emitter.constLits[constIdx]!;
    } else {
        name = toString(constName, undefined);
        lit = toJsLiteral(name);
    }
    const val = `g${globals.size + 1}`;
    const expr = `(${val} === undefined ? (${val} = global.get(${lit})) : ${val})`;
    const desc: GlobalDesc = { v: val, l: lit, e: expr, n: name };
    globals.set(constIdx, desc);
    return desc;
}
