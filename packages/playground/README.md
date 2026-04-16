# @mirascript/playground

`@mirascript/playground` 是 MiraScript 的在线示例与调试界面，基于 Vite、Monaco Editor 和工作区内的语言包构建。

## 功能

- 在线编辑 MiraScript 与 MiraScript Template
- 内置示例切换
- 编译结果与运行结果并排查看
- 支持快捷键运行与模式切换

## 本地开发

```bash
pnpm --filter @mirascript/playground start
```

默认使用 Vite 本地开发服务器。

## 构建

```bash
pnpm --filter @mirascript/playground build
pnpm --filter @mirascript/playground serve
```

## 依赖关系

该包依赖以下工作区包：

- `@mirascript/bindings`
- `@mirascript/mirascript`
- `@mirascript/monaco`

因此在开发前，需要先完成整个工作区的依赖安装与必要构建。
