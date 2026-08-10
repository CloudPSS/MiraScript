# PreferIfExpression

等级：**提示**

更推荐使用 if 表达式而非条件表达式

## 如何修复

用 `if` 表达式替代 `?:`，使分支结构更明确。

### 修改前

```mira
let label = ready ? "yes" : "no";
```

### 修复后

```mira
let label = if ready { "yes" } else { "no" };
```
