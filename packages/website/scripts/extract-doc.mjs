/* eslint-disable no-console */
/* eslint-disable unicorn/prefer-single-call */
import { compileSync, getVmType } from '@mirascript/mirascript';
import { lib, formatDiagnosticMessage } from '@mirascript/mirascript/subtle';
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

const outDir = resolve(import.meta.dirname, '../../../docs/lib');
mkdirSync(outDir, { recursive: true });

// ── helpers ─────────────────────────────────────────────────────────────

/**
 * 格式化函数签名
 * @param {string} name
 * @param {object} entry
 * @returns {string}
 */
function formatSignature(name, entry) {
  const params = entry.params ? Object.keys(entry.params) : [];
  return `fn ${name}(${params.join(', ')})`;
}

/**
 * 格式化参数类型标注  `name: type — description`
 * @param {object} paramInfo
 * @param {number} indent
 * @returns {string}
 */
function paramDescStr(paramInfo, indent) {
  if (!paramInfo) return '';
  const typeStr = paramInfo.type ? `: \`${paramInfo.type}\`` : '';
  const descStr = paramInfo.description ? ` — ${paramInfo.description}` : '';
  const str = `${typeStr}${descStr}`;
  if (!indent) return str;
  return str.replaceAll('\n', `\n${' '.repeat(indent)}`);
}

/**
 * 生成弃用警告 Markdown
 * @param {{use?: string, message: DiagnosticCode }} deprecated
 * @returns {string[]}
 */
function renderDeprecated(deprecated) {
  const lines = [];
  lines.push(`:::warning[已弃用]`);
  lines.push(formatDiagnosticMessage(deprecated.message, deprecated.use));
  lines.push(':::');
  lines.push('');
  return lines;
}

/**
 * 生成示例
 * @param {string} example
 * @returns {string[]}
 */
function renderExample(example) {
  const lines = [];
  let fence = '```';
  while (example.includes(fence)) {
    fence += '`';
  }
  lines.push(`${fence}mira`);
  try {
    compileSync(example);
  } catch (e) {
    console.error(`示例代码编译错误：\n${example}`);
  }
  lines.push(example);
  lines.push(fence);
  return lines;
}

/**
 * 生成一个函数条目的 Markdown
 * @param {string} name
 * @param {object} entry
 * @returns {string}
 */
function renderFunction(name, entry) {
  const lines = [];
  const sig = formatSignature(name, entry);

  lines.push(`### \`${sig}\``);
  lines.push('');

  if (entry.deprecated) {
    lines.push(...renderDeprecated(entry.deprecated));
  }

  if (entry.summary) {
    lines.push(entry.summary);
    lines.push('');
  }

  // 参数
  if (entry.params && Object.keys(entry.params).length > 0) {
    lines.push('**参数**');
    lines.push('');
    for (const [pName, pDesc] of Object.entries(entry.params)) {
      lines.push(`- \`${pName}\`${paramDescStr(pDesc, 2)}`);
    }
    lines.push('');
  }

  // 返回值
  if (entry.returns) {
    lines.push(`**返回值**${paramDescStr(entry.returns, 0)}`);
    lines.push('');
  }

  // 示例
  if (entry.examples?.length) {
    lines.push('**示例**');
    lines.push('');
    for (const ex of entry.examples) {
      lines.push(...renderExample(ex));
      lines.push('');
    }
  }

  return lines.join('\n');
}

/**
 * 生成一个常量条目的 Markdown
 * @param {string} name
 * @param {object} entry
 * @returns {string}
 */
function renderConst(name, entry) {
  const lines = [];
  lines.push(`### \`${name}\`: \`${getVmType(entry.value)}\``);
  lines.push('');

  if (entry.deprecated) {
    lines.push(...renderDeprecated(entry.deprecated));
  }

  if (entry.summary) {
    lines.push(entry.summary);
    lines.push('');
  }
  return lines.join('\n');
}

// ── 分类条目 ────────────────────────────────────────────────────────────

/** @type {Array<{name: string, entry: object, kind: 'function'|'const'}>} */
const globals = [];
/** @type {Array<{name: string, entries: Record<string, object>}>} */
const modules = [];

for (const [name, value] of Object.entries(lib)) {
  if (typeof value === 'object' && value != null && !('value' in value) && !('summary' in value)) {
    // 模块命名空间
    modules.push({ name, entries: value });
  } else if (typeof value === 'function') {
    globals.push({ name, entry: value, kind: 'function' });
  } else {
    globals.push({ name, entry: value, kind: 'const', value: value?.value });
  }
}

globals.sort((a, b) => {
  if (a.kind !== b.kind) {
    return a.kind === 'function' ? 1 : -1;
  }

  return a.name.localeCompare(b.name, 'en', { sensitivity: 'base' });
});

// ── 模块编号映射 ────────────────────────────────────────────────────────

const moduleIndex = new Map();
for (const [i, mod] of modules.entries()) {
  const num = String((i + 1) * 10).padStart(2, '0');
  moduleIndex.set(mod.name, num);
}

// ── 生成 00-global.md ───────────────────────────────────────────────────

{
  const lines = [];
  lines.push('# 全局');
  lines.push('');

  for (const g of globals) {
    if (g.kind === 'function') {
      lines.push(renderFunction(g.name, g.entry));
    } else {
      lines.push(renderConst(g.name, g.entry));
    }
  }

  // 模块交叉引用
  for (const mod of modules) {
    const num = moduleIndex.get(mod.name);
    lines.push(`## 模块 \`${mod.name}\``);
    lines.push('');
    lines.push(`参见 [模块 ${mod.name}](./${num}-${mod.name}.md)。`);
    lines.push('');
  }

  writeFileSync(resolve(outDir, '00-global.md'), lines.join('\n'));
  console.log('写入 docs/lib/00-global.md');
}

// ── 生成模块文件 ────────────────────────────────────────────────────────

for (const mod of modules) {
  const num = moduleIndex.get(mod.name);
  const lines = [];
  lines.push(`# 模块 \`${mod.name}\``);
  lines.push('');

  for (const [name, value] of Object.entries(mod.entries)) {
    if (typeof value === 'function') {
      lines.push(renderFunction(name, value));
    } else {
      lines.push(renderConst(name, value));
    }
  }

  const fileName = `${num}-${mod.name}.md`;
  writeFileSync(resolve(outDir, fileName), lines.join('\n'));
  console.log(`写入 docs/lib/${fileName}`);
}
