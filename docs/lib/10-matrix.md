# 模块 `matrix`

### `fn add(a, b)`

逐项相加

**参数**

- `a`: `number | number[] | number[][]`: 第一个操作数
- `b`: `number | number[] | number[][]`: 第二个操作数

**返回值** `number | number[] | number[][]`

**示例**

```mira
matrix.add([1, 2], [3, 4]) // [4, 6]
```

### `fn diagonal(x, k)`

创建一个对角矩阵或获取矩阵的对角线

**参数**

- `x`: `number[] | number[][]`: 对角线元素或要获取对角线的矩阵
- `k`: `number`: 对角线偏移量，默认为 0

**返回值** `number[][] | number[]`

**示例**

```mira
matrix.diagonal([1, 2, 3]) // [[1, 0, 0], [0, 2, 0], [0, 0, 3]]
```

```mira
matrix.diagonal([[1, 2], [3, 4]]) // [1, 4]
```

### `fn entrywise(a, b, f)`

逐项操作

**参数**

- `a`: `any | any[] | any[][]`: 第一个操作数
- `b`: `any | any[] | any[][]`: 第二个操作数
- `f`: `fn(a: any, b: any) -> any`: 操作函数

**返回值** `any | any[] | any[][]`

**示例**

```mira
matrix.entrywise([1, 2], [3, 4], fn (x, y) { x + y }) // [4, 6]
```

### `fn entrywise_divide(a, b)`

逐项相除

**参数**

- `a`: `number | number[] | number[][]`: 第一个操作数
- `b`: `number | number[] | number[][]`: 第二个操作数

**返回值** `number | number[] | number[][]`

**示例**

```mira
matrix.entrywise_divide([4, 6], [2, 3]) // [2, 2]
```

### `fn entrywise_multiply(a, b)`

逐项相乘

**参数**

- `a`: `number | number[] | number[][]`: 第一个操作数
- `b`: `number | number[] | number[][]`: 第二个操作数

**返回值** `number | number[] | number[][]`

**示例**

```mira
matrix.entrywise_multiply([1, 2], [3, 4]) // [3, 8]
```

### `fn identity(..size)`

创建一个单位矩阵

**参数**

- `..size`: `[number] | [number, number]`: 矩阵的维度

**返回值** `number[][]`

**示例**

```mira
matrix.identity(3) // [[1, 0, 0], [0, 1, 0], [0, 0, 1]]
```

### `fn invert(a)`

矩阵求逆

**参数**

- `a`: `number | number[][]`: 待求逆的矩阵

**返回值** `number | number[][]`

**示例**

```mira
matrix.invert([[1, 2], [3, 4]]) // [[-2, 1], [1.5, -0.5]]
```

### `fn multiply(a, b)`

矩阵相乘

**参数**

- `a`: `number | number[] | number[][]`: 第一个操作数
- `b`: `number | number[] | number[][]`: 第二个操作数

**返回值** `number | number[] | number[][]`

**示例**

```mira
matrix.multiply([[1, 2], [3, 4]], [5, 6]) // [17, 39]
```

### `fn ones(..size)`

创建一个全一的矩阵

**参数**

- `..size`: `number[]`: 矩阵的维度

**返回值** `number[][]`

**示例**

```mira
matrix.ones(2, 2) // [[1, 1], [1, 1]]
```

### `fn size(matrix)`

获取矩阵尺寸

**参数**

- `matrix`: `any[][]`: 要获取尺寸的矩阵

**返回值** `[number, number]`

**示例**

```mira
matrix.size([[1, 2], [3, 4]]) // [2, 2]
```

### `fn subtract(a, b)`

逐项相减

**参数**

- `a`: `number | number[] | number[][]`: 第一个操作数
- `b`: `number | number[] | number[][]`: 第二个操作数

**返回值** `number | number[] | number[][]`

**示例**

```mira
matrix.subtract([3, 4], [1, 2]) // [2, 2]
```

### `fn transpose(matrix)`

转置矩阵

**参数**

- `matrix`: `any[][]`: 要转置的矩阵

**返回值** `any[][]`

**示例**

```mira
matrix.transpose([[1, 2], [3, 4]]) // [[1, 3], [2, 4]]
```

### `fn zeros(..size)`

创建一个全零的矩阵

**参数**

- `..size`: `number[]`: 矩阵的维度

**返回值** `number[][]`

**示例**

```mira
matrix.zeros(2, 3) // [[0, 0, 0], [0, 0, 0]]
```
