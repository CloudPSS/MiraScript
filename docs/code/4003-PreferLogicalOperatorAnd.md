# PreferLogicalOperatorAnd

等级：**提示**

逻辑运算中更推荐使用 `&&` 而非 `and`

## 如何修复

将模式关键字风格的 `and` 改为逻辑运算符 `&&`。

### 修改前

```mira
let ready = loaded and valid;
```

### 修复后

```mira
let ready = loaded && valid;
```
