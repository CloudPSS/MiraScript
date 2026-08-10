# UnnecessaryIrrefutablePattern

等级：**警告**

不可失败匹配中的该模式是多余的；请考虑移除它或改为在 `is` 表达式中使用

## 如何修复

绑定模式总能成功时，不必放在 `is` 表达式中，直接声明变量即可。

### 修改前

```mira
let matched = value is captured;
```

### 修复后

```mira
let captured = value;
```
