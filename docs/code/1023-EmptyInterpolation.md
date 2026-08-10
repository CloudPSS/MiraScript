# EmptyInterpolation

等级：**错误**

插值表达式为空

## 如何修复

在插值括号内提供表达式，或删除空插值。

### 修改前

```mira
let text = "value: $()";
```

### 修复后

```mira
let value = 42;
let text = "value: $value";
```
