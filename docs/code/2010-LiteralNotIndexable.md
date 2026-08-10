# LiteralNotIndexable

等级：**警告**

字面量不能作为记录或数组访问

## 如何修复

只有数组、记录等可索引值才能使用字段或索引访问。

### 修改前

```mira
let value = 42[0];
```

### 修复后

```mira
let value = [42][0];
```
