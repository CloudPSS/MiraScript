# PreferLogicalOperatorNot

等级：**提示**

逻辑运算中更推荐使用 `!` 而非 `not`

## 如何修复

将模式关键字风格的 `not` 改为逻辑运算符 `!`。

### 修改前

```mira
let hidden = not visible;
```

### 修复后

```mira
let hidden = !visible;
```
