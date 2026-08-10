# DuplicateSpreadPattern

等级：**错误**

数组模式中展开模式只能使用一次

## 如何修复

一个数组模式只能有一个展开模式；将其余部分合并到同一个绑定中。

### 修改前

```mira
let [..head, middle, ..tail] = values;
```

### 修复后

```mira
let [first, ..rest] = values;
```
