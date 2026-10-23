# OverflowIntegerLiteral

等级：**错误**

整数字面量过大

## 如何修复

将整数控制在可表示范围内，或改用十进制（浮点）表示。

### 修改前

```mira
let value = 0xFFFFFFFFFFFFFFFFFFFFFFFF;
```

### 修复后

```mira
let value = 7.922_816_251_426_434e+28;
```
