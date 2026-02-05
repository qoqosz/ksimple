# k/simple

A tiny k interpreter for educational purposes, based on the C code with comments https://github.com/kparc/ksimple/.

## Build and Run

No dependencies required, only standard Rust.

```bash
cargo run
```

## Test

```bash
diff -u --label actual --label expected <(cargo run test/t.k 2>/dev/null) test/t.out
```

## Usage

```bash
$ cargo run
k/simple in Rust
k)2+2
4
k)x:!9
k)y:2+x
k)x-y
-2 -2 -2 -2 -2 -2 -2 -2 -2 
k)z:x,y
k)#z
18
k)x+!3
dyadic_add:169 domain

Error
k)\w
288
k)x:y:z:0
k)\w
0
k)^C
```

## More examples

Numbers are 64-bit integers

```
k)257*-257
-66049
```

Operation order

```
k)2+3*4
14
```

Simple operations on vectors

```
k)x:!9
k)1+x
1 2 3 4 5 6 7 8 9 
k)4=x
0 0 0 0 1 0 0 0 0 
```

Sum of squares

```
k)x:1+!9
k)+/x*x
204
```

Factorial

```
k)x:1+!9
k)*\x
1 2 6 24 120 720 5040 40320 362880 
```

## Code structure

See [IMPLEMENTATION.md](IMPLEMENTATION.md) for more details..

- `src/lib/token.rs`: Tokenizer
- `src/lib/value.rs`: Values and vector operations
- `src/lib/runtime.rs`: Runtime environment and verb implementations
- `src/lib/repl.rs`: Expression evaluation and REPL
- `src/lib/main.rs`: Main entry point
