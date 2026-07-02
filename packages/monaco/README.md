# @mirascript/monaco

`@mirascript/monaco` 为 Monaco Editor 提供 MiraScript 语言支持，包括语言注册、基础能力和可扩展的语言服务特性。

## 能力概览

- 注册 `mirascript` 与 `mirascript-template` 语言
- 提供高亮、基础编辑支持与结构化语言能力
- 支持按需加载补全、诊断、跳转等能力
- 支持注入全局上下文，增强编辑体验

## 安装

```bash
pnpm add @mirascript/monaco monaco-editor
```

## 快速开始

```ts
import * as monaco from 'monaco-editor';
import { registerMiraScript } from '@mirascript/monaco';

registerMiraScript(monaco);
```

注入全局信息示例：

```ts
registerMiraScript(monaco, () => ({
  globals: {
    PI: 3.14159,
  },
}));
```

## 模块入口

- `@mirascript/monaco`：默认入口
- `@mirascript/monaco/basic`：仅基础语言支持
- `@mirascript/monaco/lsp`：语言服务相关能力

## 开发

```bash
pnpm --filter @mirascript/monaco build
```
