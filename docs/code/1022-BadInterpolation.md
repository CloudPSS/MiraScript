# BadInterpolation

等级：**错误**

无效的插值表达式

## 如何修复

确保插值中是完整的表达式；复杂逻辑可先计算到变量中。

### 修改前

```mira
let text = "total: $(1 + )";
```

### 修复后

```mira
let total = 1 + 2;
let text = "total: $total";
```
