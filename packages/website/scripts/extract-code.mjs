/* eslint-disable no-console */
/* eslint-disable unicorn/prefer-single-call */
import { DiagnosticCode, getDiagnosticMessage, getDiagnosticSeverity } from '@mirascript/mirascript/subtle';
import { existsSync, mkdirSync, readdirSync, writeFileSync } from 'node:fs';
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
  if (typeof code !== 'number') continue;
  const msg = getDiagnosticMessage(code);
  const sev = getDiagnosticSeverity(code);
  if (!msg || !sev) continue;
  entries.push({ code, name, message: msg, severity: sev });
}

entries.sort((a, b) => a.code - b.code);

// ── 生成文件 ────────────────────────────────────────────────────────────

/** @type {Set<string>} */
const writtenFiles = new Set();

for (const { code, name, message, severity: sev } of entries) {
  const fileName = `${code}-${name}.md`;
  const filePath = resolve(outDir, fileName);
  writtenFiles.add(fileName);

  if (existsSync(filePath)) {
    continue;
  }

  const lines = [];
  lines.push(`# ${name}`);
  lines.push('');
  lines.push(`**${severityLabel(sev)}**`);
  lines.push('');
  lines.push(message);
  lines.push('');

  writeFileSync(filePath, lines.join('\n'));
  console.log(`写入 docs/code/${fileName}`);
}

// ── _category_.json ─────────────────────────────────────────────────────

const categoryFile = '_category_.json';
writtenFiles.add(categoryFile);
writeFileSync(
  resolve(outDir, categoryFile),
  JSON.stringify({ label: '诊断代码', position: 3, link: { type: 'generated-index', slug: '/code/' } }, null, 2) + '\n',
);

// ── 检查多余文件 ────────────────────────────────────────────────────────

const existing = readdirSync(outDir);
const extra = existing.filter((f) => !writtenFiles.has(f));
if (extra.length > 0) {
  console.warn(`\n⚠ docs/code/ 下存在多余文件：`);
  for (const f of extra) {
    console.warn(`  - ${f}`);
  }
}
