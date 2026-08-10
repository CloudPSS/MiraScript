# OverflowIntegerLiteral

等级：**错误**

整数字面量过大

## 如何修复

将整数控制在可表示范围内，或改用较小的单位。

### 修改前

```mira
let mask = 0xFFFFFFFFFFFFFFFFFFFFFFFF;
```

### 修复后

```mira
let mask = 0xFFFFFFFF;
```
