# @mirascript/playground

`@mirascript/playground` 是 MiraScript 的在线编辑与调试界面，基于 Vite 与 Monaco 构建。

## 能力概览

- 在线编辑 MiraScript 与模板脚本
- 内置示例切换
- 展示编译与运行结果
- 支持交互式调试体验

## 本地开发

```bash
pnpm --filter @mirascript/playground start
```

## 构建与预览

```bash
pnpm --filter @mirascript/playground build
pnpm --filter @mirascript/playground serve
```

## 依赖

该包主要依赖以下工作区包：

- `@mirascript/bindings`
- `@mirascript/mirascript`
- `@mirascript/monaco`
