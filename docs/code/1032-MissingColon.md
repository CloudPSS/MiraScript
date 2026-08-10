# MissingColon

等级：**错误**

条件表达式中缺少 `:`

## 如何修复

条件表达式的两个结果之间需要 `:`；也可以改写为更清晰的 `if` 表达式。

### 修改前

```mira
let label = true ? "yes" "no";
```

### 修复后

```mira
let label = if true { "yes" } else { "no" };
```
