# MispositionedSpreadInRecordPattern

等级：**错误**

记录模式中的展开模式应为最后一个字段

## 如何修复

记录模式的展开字段必须放在最后。

### 修改前

```mira
let (..rest, name: name) = person;
```

### 修复后

```mira
let (name: name, ..rest) = person;
```
