# UnusedLocalVariable

等级：**提示**

局部变量未使用；请考虑删除它或使用 `_` 忽略

## 如何修复

删除未使用的变量、实际读取它，或用 `_` 明确忽略结果。

### 修改前

```mira
let result = compute();
42
```

### 修复后

```mira
let result = compute();
result
```
