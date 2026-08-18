/* eslint-disable no-console */
/* eslint-disable unicorn/prefer-single-call */
import { DiagnosticCode, getDiagnosticMessage, getDiagnosticSeverity } from '@mirascript/mirascript/subtle';
import { mkdirSync, readdirSync, renameSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

const outDir = resolve(import.meta.dirname, '../../../docs/code');
mkdirSync(outDir, { recursive: true });

// ── 诊断严重程度分类 ────────────────────────────────────────────────────

/**
 * 格式化严重程度标签
 * @param {string} s
 * @returns {string}
 */
function severityLabel(s) {
  switch (s) {
    case 'error':
      return '错误';
    case 'warning':
      return '警告';
    case 'info':
      return '信息';
    case 'hint':
      return '提示';
    default:
      return s;
  }
}

// ── 收集所有有消息的诊断代码 ────────────────────────────────────────────

/**
 * @type {Array<{code: number, name: string, message: string, severity: string}>}
 */
const entries = [];

for (const [name, code] of Object.entries(DiagnosticCode)) {
  if (typeof code !== 'number' || code === DiagnosticCode.Unknown) continue;
  const msg = getDiagnosticMessage(code);
  const sev = getDiagnosticSeverity(code);
  if (!msg || !sev) continue;
  entries.push({ code, name, message: msg, severity: sev });
}

entries.sort((a, b) => a.code - b.code);

// ── 按 codename 索引现有文档 ────────────────────────────────────────────

const docFilePattern = /^\d+-(?<codename>[A-Z][A-Za-z0-9]*)\.md$/u;

/** @type {Map<string, string>} */
const existingDocs = new Map();

for (const fileName of readdirSync(outDir)) {
  const codename = docFilePattern.exec(fileName)?.groups?.['codename'];
  if (!codename) continue;

  const duplicate = existingDocs.get(codename);
  if (duplicate) {
    throw new Error(`诊断代码 ${codename} 存在多个文档：${duplicate}、${fileName}`);
  }
  existingDocs.set(codename, fileName);
}

// ── 生成文件 ────────────────────────────────────────────────────────────

/** @type {Set<string>} */
const writtenFiles = new Set();

for (const { code, name, message, severity: sev } of entries) {
  const fileName = `${code}-${name}.md`;
  const existingFile = existingDocs.get(name);
  if (existingFile) {
    writtenFiles.add(fileName);
    if (existingFile === fileName) {
      console.log(`保留 docs/code/${fileName}`);
    } else {
      renameSync(resolve(outDir, existingFile), resolve(outDir, fileName));
      console.log(`重命名 docs/code/${existingFile} -> docs/code/${fileName}`);
    }
    continue;
  }

  const filePath = resolve(outDir, fileName);
  writtenFiles.add(fileName);

  const lines = [];
  lines.push(`# ${name}`);
  lines.push('');
  lines.push(`等级：**${severityLabel(sev)}**`);
  lines.push('');
  lines.push(message);
  lines.push('');

  writeFileSync(filePath, lines.join('\n'), { flag: 'wx' });
  console.log(`写入 docs/code/${fileName}`);
}

// ── 额外的文件 ─────────────────────────────────────────────────────

writtenFiles.add('_category_.json');

// ── 检查多余文件 ────────────────────────────────────────────────────────

const existing = readdirSync(outDir);
const extra = existing.filter((f) => !writtenFiles.has(f));
if (extra.length > 0) {
  console.warn(`\n⚠ docs/code/ 下存在多余文件：`);
  for (const f of extra) {
    console.warn(`  - ${f}`);
  }
}
