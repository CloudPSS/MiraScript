---
name: mirascript-cli
description: 使用官方 MiraScript CLI 查询内置手册，以及运行、验证、调试和格式化 MiraScript 脚本或模板。需要查询关键字、操作符或标准库，执行 .mira/.miratpl 文件，验证生成的代码片段，注入变量，检查 CLI 错误，进入 REPL，或安全地预览及写回格式化结果时使用。
---

# 使用 MiraScript CLI

## 选择调用方式

优先使用目标项目已固定的版本：

```powershell
pnpm exec mirascript --version
```

按以下顺序选择命令：

1. 在已安装 `@mirascript/cli` 的 pnpm 项目中使用 `pnpm exec mirascript`。
2. 已有全局或环境提供的二进制时使用 `mirascript`。
3. 项目未安装 CLI 且允许联网下载时，使用 `pnpm dlx @mirascript/cli` 或 `npx --package=@mirascript/cli mirascript`。不要仅为验证代码擅自修改项目依赖。

后文用 `pnpm exec mirascript` 展示命令；若选择了其他入口，保持子命令和参数不变。在自动化流程中始终传入脚本、`--eval` 或子命令，避免无意进入交互式 REPL。

## 运行与验证

运行脚本文件：

```powershell
pnpm exec mirascript run script.mira
```

`run` 是默认子命令，因此也可写 `pnpm exec mirascript script.mira`。显式写出 `run` 可让自动化日志更清楚。

对不含复杂 shell 字符的短代码使用 `--eval`：

```powershell
pnpm exec mirascript run --eval 'let x = 1 + 2; x'
```

对多行代码、插值或复杂引号优先使用 stdin。PowerShell 使用单引号 here-string，防止 PowerShell 展开 MiraScript 的 `$`：

```powershell
@'
fn factorial(n) {
  if n <= 1 { 1 } else { n * factorial(n - 1) }
}
factorial(5)
'@ | pnpm exec mirascript run -
```

POSIX shell 使用带引号的 heredoc：

```sh
pnpm exec mirascript run - <<'MIRA'
let values = [1, 2, 3];
values::map(fn { it * 2 })
MIRA
```

脚本模式会在 stdout 打印最后表达式的值，`debug_print` 会产生额外输出。验证时同时检查退出码、stderr 和预期 stdout；不要只检查“命令有输出”。

### 常用运行选项

| 选项                         | 用途                                                     |
| ---------------------------- | -------------------------------------------------------- |
| `-e, --eval <script>`        | 执行内联脚本；不可同时传脚本路径                         |
| `-v, --variable <key=value>` | 注入全局变量；可重复使用，值优先按 MiraScript 表达式解析 |
| `-t, --template`             | 对 `--eval`、stdin 或需强制覆盖的输入启用模板模式        |
| `--no-template`              | 即使文件名为 `.miratpl` 也强制使用脚本模式               |
| `--timeout <ms>`             | 设置检查点超时；默认 3000，`0` 表示不超时                |

```powershell
pnpm exec mirascript run script.mira --variable "name='Mira'" --variable 'count=3'
pnpm exec mirascript run --template --eval 'Hello, $name!' --variable "name='World'"
```

运行文件时会根据 `.miratpl` 扩展名自动选择模板模式；对 stdin 和 `--eval` 必须显式传 `--template`。仅对可信且确定会终止的代码使用 `--timeout 0`。

## 查询内置手册

不确定当前 CLI 版本支持的关键字、操作符或标准库接口时，先查询随 CLI 发布的手册：

```powershell
pnpm exec mirascript man keywords
pnpm exec mirascript man operators
pnpm exec mirascript man libraries
```

使用具体主题获取详细说明。关键字和操作符主题会显示语义与示例；标准库函数会显示摘要、参数、返回值和可用示例，模块主题会列出其成员：

```powershell
pnpm exec mirascript man if
pnpm exec mirascript man '&&'
pnpm exec mirascript man debug_print
pnpm exec mirascript man matrix
pnpm exec mirascript man matrix.add
```

标准库成员使用点路径；可选的 `lib.` 前缀也会被接受，如 `lib.matrix.add`。通常省略该前缀，以保持命令简洁。在 PowerShell 中始终给操作符主题加引号，避免 `&&` 等字符被 shell 解释。

`man` 只接受一个必填主题。未知主题会打印该子命令的帮助并以非零状态退出；自动化查询仍需同时检查退出码、stdout 和 stderr。

## 格式化

先预览，再决定是否写回：

```powershell
pnpm exec mirascript format script.mira
pnpm exec mirascript format --write script.mira
```

不带 `--write` 时，文件输入的格式化结果写到 stdout，并带 `// File: ...` 标题；不会执行专用的 check 模式。从 stdin 格式化时只输出格式化后的代码：

```powershell
@'
let x=1+2;
x
'@ | pnpm exec mirascript format -
```

为模板格式化显式传 `--template`：

```powershell
pnpm exec mirascript format --template template.miratpl
```

当前 `format` 实现不会像 `run` 那样根据 `.miratpl` 文件名切换输入模式。对多个文件或 glob，把模式作为单个参数交给 CLI，并在 `--write` 后检查实际改动：

```powershell
pnpm exec mirascript format --write 'src/**/*.mira'
```

不要对超出任务范围的宽泛 glob 使用 `--write`。格式化时还要检查 stderr 和匹配到的文件；无匹配项或个别文件格式化失败时，退出码本身不足以证明全部成功。

## REPL

仅在用户需要交互探索时运行：

```powershell
pnpm exec mirascript repl
```

没有脚本路径和 `--eval` 的默认 `run` 也会进入 REPL。不要在非交互验证或 CI 中这样调用。

## 错误处理与完成标准

- 退出码 `0` 表示命令进程成功；运行时编译或执行错误通常为 `2`，权限错误为 `3`，其他访问、内部或 stdin 格式化错误可能为 `1`。多文件格式化可能只记录单个文件的失败，因此仍要检查 stderr。
- 根据错误中的文件名、行列和诊断信息修复代码，并重新执行同一代表性输入。
- 对 `--variable` 解析、模板渲染、超时和宿主全局等行为分别保留针对性用例。
- 只有在退出码、stderr 和实际输出都符合预期后，才报告运行验证通过。
- 格式化写回后检查 diff，确认只修改了预期文件且语义未改变。
