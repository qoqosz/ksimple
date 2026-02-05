# Implementation Summary

This project is a compact, idiomatic Rust translation of the `k/simple` reference interpreter. The code keeps the original model of atoms, vectors, verbs/adverbs, and the right-to-left evaluator, while using explicit names and Rust data structures for clarity and safety.

## Core Concepts

- **Value model**
  - `Value::Atom(i64)` represents a scalar integer.
  - `Value::Vector(Rc<Vec<i64>>)` wraps vector data with reference counting.
  - `Value::Error` is the explicit error sentinel used throughout evaluation.

- **Vectors and workspace**
  - Vectors are stored as `Vec<i64>` inside `Rc`, replacing the manual handle/refcount heap.
  - The `\w` command sums unique vector allocations referenced by globals.

- **Tokenizer**
  - `tokenize_line` converts an input line into tokens:
    - numbers (multi-digit, signed)
    - globals (`a`..`z`)
    - symbols (verbs/adverbs)
    - `:` for assignment
  - This enables parsing of expressions like `128*2` and `-12+-3` without ambiguity.

- **Evaluation model**
  - `evaluate_expression` is a right-to-left evaluator that mirrors the reference C logic.
  - It supports monadic and dyadic verbs, adverbs, and inline global assignments (e.g. `a:7`).
  - Verb/adverb dispatch is via static tables:
    - `MONADIC_VERBS`, `DYADIC_VERBS`, `ADVERBS`

- **Verbs and adverbs**
  - Verbs implement the same semantics as the C reference, adapted for `i64`:
    - monadic: negate, enumerate, count, enlist, reverse, first
    - dyadic: add, subtract, modulo, take, concatenate, index-at, equal, not-equal, and, or, product
  - Adverbs:
    - `/` (over) folds a vector using a dyadic verb
    - `\` (scan) produces intermediate fold results

- **Error handling**
  - Errors are represented by `Value::Error`, not a numeric sentinel, so all integer values are valid input/output.
  - The `Runtime` helpers `rank_error`, `domain_error`, `length_error`, and `parse_error` emit human-readable messages consistent with the reference behavior.

## REPL Internals

The REPL is primarily defined by `run_repl` and `process_line`. It mimics the reference implementation’s flow while using Rust’s standard IO.

### `run_repl`

- Continuously prints the `k)` prompt.
- Reads a full line from stdin using `read_line`.
- Delegates handling of the line to `process_line`.
- Stops when `process_line` returns `false` or on EOF.

### `process_line`

`process_line` encapsulates all line-level logic and mirrors the C interpreter’s read-eval-print loop semantics:

1. **Ignore empty-or-whitespace lines**
   - The line is only trimmed on the right (`trim_end`).
   - Whitespace-only input ends up as an empty token list and is ignored.

2. **Handle backslash commands**
   - If the line is exactly two characters and starts with `\`:
     - `\\` exits the REPL (`false`)
     - `\w` prints the workspace byte count
     - `\v` prints global variables with refcounts and vector lengths

3. **Handle comments**
   - If the line starts with `/`, it is ignored and the REPL continues.

4. **Tokenize**
   - `tokenize_line` converts the line into tokens, handling multi-digit and signed integers.
   - A tokenization error yields a parse error message and continues the REPL.

5. **Evaluate**
   - `evaluate_expression` performs right-to-left evaluation of the tokens.
   - The evaluator supports adverbs, monadic verbs, dyadic verbs, and inline assignment.

6. **Assignment suppression**
   - If the expression is a global assignment (`a:...`), output is suppressed to match the C behavior.

7. **Print**
   - Non-assignment results are pretty-printed.

This separation keeps the REPL loop minimal while concentrating all parsing and evaluation detail inside `process_line` and `evaluate_expression`.
