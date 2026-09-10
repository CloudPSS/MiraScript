---
sidebar_position: 15
---

# 复数计算

`complex` 模块使用普通 record `(实部, 虚部)` 表示复数，不引入新的值类型，也不改变 `+`、`*` 等运算符。

```mira
complex.from(3)                  // (3, 0)
complex.from(('3', '4'))         // (3, 4)
complex.add(1, 2)                // (3, 0)
complex.multiply((1, 2), (3, 4)) // (-5, 10)
complex.sqrt(-4)                 // (0, 2)
complex.abs((3, 4))              // 5
complex.polar(2, 0)              // (2, 0)
```

## 输入与结果

所有复数参数都使用与 `complex.from(value)` 相同的转换规则：

- record 同时包含 `0`、`1` 字段时，分别按 `to_number` 语义转换；其他字段忽略。
- 其他输入整体按 `to_number` 语义转换，并补上虚部 `+0`。数组不会作为复数二元组解析。
- 字段存在但值为 `nil` 时仍尝试转换，转换失败会抛出运行时错误。缺参、非法字符串和不能转换的对象同样报错。

复数结果始终是仅含 `0`、`1` 两个 number 字段的 record。`real`、`imag`、`abs` 和 `arg` 返回 number。

## 成员

| 类别         | 成员                                                                  |
| ------------ | --------------------------------------------------------------------- |
| 常量与转换   | `I`、`from(value)`                                                    |
| 算术         | `add(a,b)`、`subtract(a,b)`、`multiply(a,b)`、`divide(a,b)`、`neg(z)` |
| 幂根         | `pow(a,b)`、`sqrt(z)`、`cbrt(z)`                                      |
| 指数与对数   | `exp(z)`、`expm1(z)`、`log(z)`、`log1p(z)`、`log2(z)`、`log10(z)`     |
| 三角与逆三角 | `sin(z)`、`cos(z)`、`tan(z)`、`asin(z)`、`acos(z)`、`atan(z)`         |
| 双曲与逆双曲 | `sinh(z)`、`cosh(z)`、`tanh(z)`、`asinh(z)`、`acosh(z)`、`atanh(z)`   |
| 帮助函数     | `real(z)`、`imag(z)`、`abs(z)`、`arg(z)`、`conj(z)`、`polar(r,theta)` |

`I` 为 `(0,1)`。`polar` 的两个参数均按普通数字转换，角度单位为弧度，结果为 `(r*cos(theta), r*sin(theta))`。`arg(z)` 返回主辐角，范围为 `[-PI, PI]`。

多值函数返回主值。例如 `cbrt(-8)` 约为 `(1,1.7320508075688772)`。分支切线保留符号零：`sqrt((-4,0))` 为 `(0,2)`，`sqrt((-4,-0))` 为 `(0,-2)`；`log((-1,-0))` 为 `(0,-PI)`。

NaN 和 Infinity 是合法数值。计算奇点不会抛出转换错误，例如 `log(0)` 为 `(-inf,0)`，`divide(0,0)` 为 `(nan,nan)`。零次幂为 `(1,0)`，包括底数含 NaN 的情况。一般浮点计算允许舍入误差，应逐分量使用合适的容差比较。

本模块暂不提供取整、聚合、gamma、factorial、sign、min/max、atan2、random 或 `to_polar`。
