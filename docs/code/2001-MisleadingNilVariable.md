# MisleadingNilVariable

等级：**警告**

`$0` 不是空值；显式使用全局变量 `global.$0` 或空值 `nil`

## 如何修复

空值写作 `nil`；若确实要读取同名全局变量，请显式使用 `global.<name>`。

### 修改前

```mira
let fallback = null;
```

### 修复后

```mira
let fallback = nil;
```
