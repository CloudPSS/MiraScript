# @mirascript/monaco

`@mirascript/monaco` 为 Monaco Editor 提供 MiraScript 语言支持，包括语言注册、基础编辑体验以及基于语言服务的增强能力。

## 功能

- 注册 `mirascript` 与 `mirascript-template` 语言
- 基础语言特性，如高亮、括号匹配、简单编辑支持
- 可按需加载的 LSP 风格能力，如补全、诊断、跳转等
- 支持注入运行时上下文信息，增强编辑体验

## 安装

```bash
pnpm add @mirascript/monaco monaco-editor
```

## 基本用法

```ts
import * as monaco from 'monaco-editor';
import { registerMiraScript } from '@mirascript/monaco';

registerMiraScript(monaco);
```

如果需要提供全局变量或上下文信息：

```ts
registerMiraScript(monaco, () => ({
  globals: {
    PI: 3.14159,
  },
}));
```

## 模块入口

- `@mirascript/monaco`：默认入口，按需加载基础功能和 LSP 功能
- `@mirascript/monaco/basic`：仅基础语言支持
- `@mirascript/monaco/lsp`：LSP 相关能力

## 开发

```bash
pnpm --filter @mirascript/monaco build
```

该包通常与 `@mirascript/mirascript`、`@mirascript/help` 和 `@mirascript/constants` 一起使用。
