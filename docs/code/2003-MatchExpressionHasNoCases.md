# MatchExpressionHasNoCases

等级：**警告**

该 `match` 表达式没有分支；它永远不会匹配任何值

## 如何修复

为空的 `match` 添加至少一个 `case`，通常还应提供兜底分支。

### 修改前

```mira
let label = match value { };
```

### 修复后

```mira
let label = match value {
  case nil { "empty" }
  case _ { "value" }
};
```
