# @mirascript/help

`@mirascript/help` 将仓库文档编译为可直接消费的帮助数据，主要用于编辑器提示、站点展示和文档检索。

## 数据来源

构建过程会读取 `docs/references` 下的文档，并基于 front-matter 中的 `token`、`title` 等字段生成产物。

## 主要导出

- `KEYWORDS`：关键字到 Markdown 文本的映射
- `OPERATORS`：运算符到 Markdown 文本的映射
- `Keyword` / `Operator`：由生成结果推导的字面量类型

## 安装

```bash
pnpm add @mirascript/help
```

## 快速开始

```ts
import { KEYWORDS, OPERATORS } from '@mirascript/help';

console.log(KEYWORDS['match']);
console.log(OPERATORS['??']);
```

## 构建

```bash
pnpm --filter @mirascript/help build
```

当 `docs/references` 内容变更后，请重新构建以刷新 `dist/`。
