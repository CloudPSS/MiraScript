# @mirascript/constants

`@mirascript/constants` 导出 MiraScript 编译器常量与辅助类型，包括关键字集合、诊断码、操作码和配置枚举。

## 主要导出

- `DiagnosticCode`
- `OpCode`
- `DIAGNOSTIC_MESSAGES`
- `KEYWORDS` 与相关关键字分类
- `Config`、`InputMode`、`DiagnosticPositionEncoding`
- `isKeyword()` 等工具函数

## 安装

```bash
pnpm add @mirascript/constants
```

## 快速开始

```ts
import { KEYWORDS, isKeyword, DiagnosticCode } from '@mirascript/constants';

console.log(KEYWORDS.includes('let'));
console.log(isKeyword('match'));
console.log(DiagnosticCode.InvalidKeyword);
```

## 构建

```bash
pnpm --filter @mirascript/constants wasm
pnpm --filter @mirascript/constants build
```

构建依赖 Rust 工具链与 `wasm-pack`。
