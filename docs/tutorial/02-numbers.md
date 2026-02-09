# 数值与运算

MiraScript 中的数字类型 `number` 使用 64 位浮点数表示，可以表示整数和小数。

## 数字字面量

### 基本写法

```mira
debug_print(42);       // 整数
debug_print(3.14);     // 小数
debug_print(0.5);      // 小数点前后都不能省略
debug_print(1_000_000); // 可以用下划线分隔，提高可读性
```

### 科学计数法

```mira
debug_print(1.5e3);    // 1500
debug_print(2.0e-4);   // 0.0002
```

### 不同进制

```mira
debug_print(0xFF);     // 十六进制，值为 255
debug_print(0o77);     // 八进制，值为 63
debug_print(0b1010);   // 二进制，值为 10
```

### 特殊数值

```mira
debug_print(inf);      // 正无穷大
debug_print(-inf);     // 负无穷大
debug_print(nan);      // 非数（Not a Number）
```

## 算术运算符

MiraScript 支持常见的算术运算：

```mira
debug_print("加法:", 3 + 5);       // 8
debug_print("减法:", 10 - 4);      // 6
debug_print("乘法:", 6 * 7);       // 42
debug_print("除法:", 15 / 4);      // 3.75
debug_print("取余:", 17 % 5);      // 2
debug_print("幂运算:", 2^10);      // 1024
```

:::info
MiraScript 使用 `^` 表示幂运算（而非异或运算）。`2^10` 就是 $2^{10} = 1024$。
:::

### 运算优先级

运算优先级从高到低：

1. **幂运算** `^`（从右到左结合）
2. **前缀** `+`、`-`（正号、负号）
3. **乘除余** `*`、`/`、`%`
4. **加减** `+`、`-`

用括号 `()` 可以改变运算顺序：

```mira
debug_print(2 + 3 * 4);     // 14，先乘后加
debug_print((2 + 3) * 4);   // 20，括号内先算
debug_print(2^3^2);          // 512，即 2^(3^2) = 2^9，幂运算从右到左
debug_print(-2^2);           // -4，先算 2^2 再取负
debug_print((-2)^2);         // 4，先取负再平方
```

## 常用数学函数

MiraScript 内置了丰富的数学函数，可以直接使用：

### 取整与舍入

```mira
debug_print("floor(3.7) =", floor(3.7));   // 3，向下取整
debug_print("ceil(3.2) =", ceil(3.2));     // 4，向上取整
debug_print("round(3.5) =", round(3.5));   // 4，四舍五入
debug_print("trunc(3.9) =", trunc(3.9));   // 3，截断小数部分
```

### 常用函数

```mira
debug_print("abs(-7) =", abs(-7));         // 7，绝对值
debug_print("sign(-3) =", sign(-3));       // -1，符号
debug_print("sqrt(16) =", sqrt(16));       // 4，平方根
debug_print("cbrt(27) =", cbrt(27));       // 3，立方根
```

### 聚合函数

```mira
debug_print("max(3, 7, 1) =", max(3, 7, 1));   // 7
debug_print("min(3, 7, 1) =", min(3, 7, 1));   // 1
debug_print("sum(1, 2, 3) =", sum(1, 2, 3));   // 6
```

### 三角函数

```mira
debug_print("sin(PI / 6) =", sin(PI / 6));     // 0.5
debug_print("cos(PI / 3) =", cos(PI / 3));     // 0.5
debug_print("atan2(1, 1) =", atan2(1, 1));     // π/4
```

### 指数与对数

```mira
debug_print("exp(1) =", exp(1));         // e ≈ 2.718...
debug_print("log(E) =", log(E));         // 1，自然对数
debug_print("log2(8) =", log2(8));       // 3
debug_print("log10(1000) =", log10(1000)); // 3
```

## 数学常量

```mira
debug_print("π =", PI);           // 3.141592653589793
debug_print("e =", E);            // 2.718281828459045
debug_print("√2 =", SQRT2);       // 1.4142135623730951
```

## 实际应用

利用数学运算，我们可以完成许多工程计算：

```mira
// 勾股定理：计算直角三角形的斜边
let a = 3;
let b = 4;
let c = sqrt(a^2 + b^2);
debug_print("直角三角形斜边:", c); // 5

// 圆的面积
let r = 5;
let area = PI * r^2;
debug_print("半径 $r 的圆面积:", area);

// 温度转换：摄氏度转华氏度
let celsius = 36.5;
let fahrenheit = celsius * 9 / 5 + 32;
debug_print("$(celsius)°C = $(fahrenheit)°F");
```

## 小结

- 数字使用 64 位浮点数，支持整数、小数、科学计数法、不同进制
- 幂运算用 `^`，从右到左结合
- 内置 `abs`、`sqrt`、`sin`、`max` 等丰富的数学函数
- 数学常量 `PI`、`E`、`SQRT2` 等可以直接使用
