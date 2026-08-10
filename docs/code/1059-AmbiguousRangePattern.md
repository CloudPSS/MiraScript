# AmbiguousRangePattern

等级：**错误**

数组模式中的范围模式应加括号

## 如何修复

数组模式中的范围需要额外括号，以区别数组范围展开。

### 修改前

```mira
value is [1..3]
```

### 修复后

```mira
value is [(1..3)]
```
