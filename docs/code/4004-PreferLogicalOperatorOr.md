# PreferLogicalOperatorOr

等级：**提示**

逻辑运算中更推荐使用 `||` 而非 `or`

## 如何修复

将模式关键字风格的 `or` 改为逻辑运算符 `||`。

### 修改前

```mira
let ready = cached or loaded;
```

### 修复后

```mira
let ready = cached || loaded;
```
