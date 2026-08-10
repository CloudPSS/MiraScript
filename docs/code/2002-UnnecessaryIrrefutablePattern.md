# UnnecessaryIrrefutablePattern

等级：**警告**

不可失败匹配中的该模式是多余的；请考虑移除它或改为在 `is` 表达式中使用

## 如何修复

对于不影响匹配结果的模式，应当使用 `is` 表达式或 `match` 表达式测试匹配结果，或者使用弃元模式 `_`。

### 修改前

```mira
let [x, 1, 2] = arr;
```

### 修复后

```mira
let succeed = arr is [x, 1, 2];
```

```mira
let [x, _, _] = arr;
```
