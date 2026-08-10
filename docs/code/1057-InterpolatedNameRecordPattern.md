# InterpolatedNameRecordPattern

等级：**错误**

记录模式中不允许插值名称

## 如何修复

记录模式的字段名必须是静态名称；动态键请在匹配后通过索引读取。

### 修改前

```mira
let (`$key`: value) = record;
```

### 修复后

```mira
let (name: value) = record;
```
