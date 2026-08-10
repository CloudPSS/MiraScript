# OverflowNumberLiteral

等级：**错误**

数字字面量过大

## 如何修复

缩小数值，或根据业务含义使用 `inf`。

### 修改前

```mira
let distance = 1e9999;
```

### 修复后

```mira
let distance = inf;
```
