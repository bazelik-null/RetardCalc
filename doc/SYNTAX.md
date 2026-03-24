# Morsel Syntax Guide

Guide to Morsel syntax and language features. This document covers the core language constructs you need
to write morsel programs.

## Table of Contents

1. [Comments](#comments)
2. [Variables and Mutability](#variables-and-mutability)
3. [Data Types](#data-types)
4. [Operators](#operators)
5. [Control Flow](#control-flow)
6. [Functions](#functions)

## Comments

Comments explain your code without affecting execution.

**Single-line comments:**

```morsel
// This is a single-line comment
let x = 5; // Comment after code
```

## Variables and Mutability

Variables store data values. By default, variables are **immutable** in morsel.

### Immutable Variables

```morsel
let x = 5;
// x = 6; // Error: cannot assign to immutable variable
```

### Mutable Variables

Use the `mut` keyword to make a variable mutable:

```morsel
let mut y = 5;
y = 6; // OK
```

### Shadowing

You can declare a new variable with the same name, shadowing the previous one:

```morsel
let x = 5;
let x = x + 1; // x is now 6
let x = x * 2; // x is now 12
```

## Data Types

Morsel is **statically typed**. The compiler can usually infer types, but you can also explicitly annotate them.

### Scalar Types

- **Integer:** `int` - 32-bit integer
- **Float:** `float` - 32-bit floating-point number
- **String:** `string` - Text data
- **Boolean:** `bool` - Boolean value (true/false)

## Operators

### Arithmetic Operators

```morsel
let a = 10;
let b = 3;

let sum = a + b; // 13
let diff = a - b; // 7
let prod = a * b; // 30
let quot = a / b; // 3 (integer division)
let rem = a % b; // 1 (remainder)
```

### Comparison Operators

```morsel
let a = 5;
let b = 3;

a == b; // false
a != b; // true
a > b;  // true
a < b;  // false
a >= b; // true
a <= b; // false
```

### Logical Operators

```morsel
let t = true;
let f = false;

t && f; // false (AND)
t || f; // true (OR)
!t;     // false (NOT)
```

## Control Flow

### if/else Expressions

```morsel
let x = 5;

if (x > 0) {
    println("x is positive");
} else if (x < 0) {
    println("x is negative");
} else {
    println("x is zero");
}
```

## Functions

Functions are declared with the `func` keyword.

### Basic Function

```morsel
func add(a: int, b: int): int {
	a + b // Implicit return
}

let result = add(5, 3); // 8
```

### Function with No Return Value

```morsel
func print_number(x: int) {
	println("The number is: ", x);
}
```

### Early Return

```morsel
func check_age(age: int): string {
	if (age < 18) {
		return "Too young";
	}
	"Old enough"
}
```