# InvalidUnicodeEscapeSequence

等级：**错误**

Unicode 转义序列的值不是有效的 Unicode 码点

## 如何修复

将转义值改为有效的 Unicode 码点。

### 修改前

```mira
let face = "\u{110000}";
```

### 修复后

```mira
let face = "\u{1F600}";
```
