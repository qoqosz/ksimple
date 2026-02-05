use crate::token::Token;
use crate::value::Value;
use std::collections::HashSet;
use std::fmt::Display;
use std::rc::Rc;

type MonadicVerb = fn(&Runtime, Value) -> Value;
type DyadicVerb = fn(&Runtime, Value, Value) -> Value;
type Adverb = fn(&Runtime, usize, Value) -> Value;

/// The runtime environment.
pub struct Runtime {
    /// The global variables: a-z.
    globals: [Value; 26],
}

/// Display the runtime environment.
impl Display for Runtime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, value) in self.globals.iter().enumerate() {
            if let Value::Vector(vector) = value {
                let name = (b'a' + index as u8) as char;
                let length = vector.len();
                let ref_count = Rc::strong_count(vector);
                writeln!(f, "{}[{}] {}", name, length, ref_count)?;
            }
        }
        Ok(())
    }
}

impl Runtime {
    /// Create a new runtime environment.
    pub fn new() -> Self {
        Self {
            globals: std::array::from_fn(|_| Value::Atom(0)),
        }
    }

    /// Get the total size of allocated memory for vectors in a workspace.
    pub(crate) fn workspace_bytes(&self) -> usize {
        let mut seen: HashSet<*const Vec<i64>> = HashSet::new();
        let mut total = 0;

        for value in &self.globals {
            if let Value::Vector(vector) = value {
                let ptr = Rc::as_ptr(vector);
                if seen.insert(ptr) {
                    total += vector.len();
                }
            }
        }

        total * std::mem::size_of::<i64>()
    }

    /// Convert a token to a value and take ownership of it.
    pub(crate) fn noun_from_token(&mut self, token: &Token) -> Value {
        match token {
            Token::Number(value) => Value::Atom(*value),
            Token::Global(name) => {
                let index = (name - b'a') as usize;
                self.globals[index].clone()
            }
            _ => Value::Error,
        }
    }

    /// Assign a value to a global variable.
    pub(crate) fn assign_global(&mut self, index: usize, value: Value) -> Value {
        self.globals[index] = value.clone();
        value
    }

    /// Report an error.
    #[track_caller]
    pub(crate) fn report_error(&self, function_name: &str, message: &str) -> Value {
        let line = std::panic::Location::caller().line();
        println!("{}:{} {}\n", function_name, line, message);
        Value::Error
    }

    #[track_caller]
    pub(crate) fn rank_error(&self, function_name: &str) -> Value {
        self.report_error(function_name, "rank")
    }

    #[track_caller]
    pub(crate) fn domain_error(&self, function_name: &str) -> Value {
        self.report_error(function_name, "domain")
    }

    #[track_caller]
    fn length_error(&self, function_name: &str) -> Value {
        self.report_error(function_name, "length")
    }

    #[track_caller]
    pub(crate) fn parse_error(&self, function_name: &str) -> Value {
        self.report_error(function_name, "parse")
    }

    #[track_caller]
    pub(crate) fn not_implemented_error(&self, function_name: &str) -> Value {
        self.report_error(function_name, "nyi")
    }
}

fn monadic_not_a_verb(runtime: &Runtime, _value: Value) -> Value {
    runtime.domain_error("monadic_not_a_verb")
}

fn dyadic_not_a_verb(runtime: &Runtime, _left: Value, _right: Value) -> Value {
    runtime.domain_error("dyadic_not_a_verb")
}

fn monadic_not_implemented(runtime: &Runtime, _value: Value) -> Value {
    runtime.not_implemented_error("monadic_not_implemented")
}

/// Negate `value`.
fn monadic_negate(_runtime: &Runtime, value: Value) -> Value {
    -value
}

/// Enumerate `value`.
fn monadic_enumerate(runtime: &Runtime, value: Value) -> Value {
    match value {
        Value::Atom(integer) => match integer {
            ..0 => runtime.domain_error("monadic_enumerate"),
            _ => (0..integer).map(|i| i as i64).collect::<Vec<_>>().into(),
        },
        Value::Vector(_) => runtime.rank_error("monadic_enumerate"),
        Value::Error => Value::Error,
    }
}

/// Return the length of `value`.
fn monadic_count(runtime: &Runtime, value: Value) -> Value {
    match value {
        Value::Atom(_) => runtime.rank_error("monadic_count"),
        Value::Vector(vector) => vector.len().into(),
        Value::Error => Value::Error,
    }
}

/// Enlist `value`.
fn monadic_enlist(runtime: &Runtime, value: Value) -> Value {
    value
        .enlist()
        .unwrap_or_else(|_| runtime.rank_error("monadic_enlist"))
}

/// Reverse `value`.
fn monadic_reverse(runtime: &Runtime, value: Value) -> Value {
    value
        .reverse()
        .unwrap_or_else(|_| runtime.rank_error("monadic_reverse"))
}

/// Return the first element of `value`.
fn monadic_first(runtime: &Runtime, value: Value) -> Value {
    dyadic_index_at(runtime, value, 0_i64.into())
}

/// Add `left` and `right`.
fn dyadic_add(runtime: &Runtime, left: Value, right: Value) -> Value {
    left.apply_dyadic_verb(&right, i64::wrapping_add)
        .unwrap_or_else(|_| runtime.domain_error("dyadic_add"))
}

/// Subtract `right` from `left`.
fn dyadic_subtract(runtime: &Runtime, left: Value, right: Value) -> Value {
    dyadic_add(runtime, left, -right)
}

/// Modulo `left` and `right`.
fn dyadic_modulo(runtime: &Runtime, left: Value, right: Value) -> Value {
    if left.is_error() || right.is_error() {
        return Value::Error;
    }

    let modulus = match left {
        Value::Atom(integer) if integer != 0 => integer,
        _ => return runtime.domain_error("dyadic_modulo"),
    };

    match right {
        Value::Atom(integer) => (integer % modulus).into(),
        Value::Vector(vector) => vector
            .iter()
            .map(|integer| integer % modulus)
            .collect::<Vec<_>>()
            .into(),
        Value::Error => Value::Error,
    }
}

/// Take the first `count` elements from `right`.
fn dyadic_take(runtime: &Runtime, left: Value, right: Value) -> Value {
    if left.is_error() || right.is_error() {
        return Value::Error;
    }

    let count = match left {
        Value::Atom(integer) if integer >= 0 => integer as usize,
        Value::Atom(_) => return runtime.domain_error("dyadic_take"),
        _ => return runtime.rank_error("dyadic_take"),
    };

    match right {
        Value::Atom(integer) => (0..count).map(|_| integer).collect::<Vec<_>>().into(),
        Value::Vector(vector) => {
            let length = vector.len();
            (0..count)
                .map(|index| vector[index.checked_rem(length).unwrap_or(0)])
                .collect::<Vec<_>>()
                .into()
        }
        Value::Error => Value::Error,
    }
}

/// Concatenate `left` and `right`.
fn dyadic_concatenate(runtime: &Runtime, left: Value, right: Value) -> Value {
    match (left, right) {
        (Value::Error, _) | (_, Value::Error) => Value::Error,
        (a @ Value::Atom(_), b @ _) => dyadic_concatenate(runtime, a.enlist().unwrap(), b),
        (a @ _, b @ Value::Atom(_)) => dyadic_concatenate(runtime, a, b.enlist().unwrap()),
        (Value::Vector(left_vector), Value::Vector(right_vector)) => left_vector
            .iter()
            .chain(right_vector.iter())
            .cloned()
            .collect::<Vec<_>>()
            .into(),
    }
}

/// Return the element at `index` in `left`.
fn dyadic_index_at(runtime: &Runtime, left: Value, right: Value) -> Value {
    if left.is_error() || right.is_error() {
        return Value::Error;
    }

    let left_vector = match left {
        Value::Vector(vector) => vector,
        Value::Atom(_) => return runtime.rank_error("dyadic_index_at"),
        Value::Error => return Value::Error,
    };

    let left_length = left_vector.len();

    match right {
        Value::Atom(index_integer) => {
            let index = index_integer as usize;

            if index_integer < 0 || index > left_length {
                return runtime.length_error("dyadic_index_at");
            }

            left_vector.get(index).copied().unwrap_or(0).into()
        }
        Value::Vector(indices) => indices
            .iter()
            .map(|index| {
                left_vector
                    .get(*index.max(&0) as usize)
                    .copied()
                    .unwrap_or(0)
            })
            .collect::<Vec<_>>()
            .into(),

        Value::Error => Value::Error,
    }
}

/// Return 1 if `left` is equal to `right`, 0 otherwise.
fn dyadic_equal(runtime: &Runtime, left: Value, right: Value) -> Value {
    left.apply_dyadic_verb(&right, |a, b| if a == b { 1 } else { 0 })
        .unwrap_or_else(|_| runtime.domain_error("dyadic_equal"))
}

/// Return 1 if `left` is not equal to `right`, 0 otherwise.
fn dyadic_not_equal(runtime: &Runtime, left: Value, right: Value) -> Value {
    left.apply_dyadic_verb(&right, |a, b| if a != b { 1 } else { 0 })
        .unwrap_or_else(|_| runtime.domain_error("dyadic_not_equal"))
}

/// Return the logical AND of `left` and `right`.
fn dyadic_and(runtime: &Runtime, left: Value, right: Value) -> Value {
    left.apply_dyadic_verb(&right, |a, b| a & b)
        .unwrap_or_else(|_| runtime.domain_error("dyadic_and"))
}

/// Return the logical OR of `left` and `right`.
fn dyadic_or(runtime: &Runtime, left: Value, right: Value) -> Value {
    left.apply_dyadic_verb(&right, |a, b| a | b)
        .unwrap_or_else(|_| runtime.domain_error("dyadic_or"))
}

/// Return the product of `left` and `right`.
fn dyadic_product(runtime: &Runtime, left: Value, right: Value) -> Value {
    left.apply_dyadic_verb(&right, |a, b| a.wrapping_mul(b))
        .unwrap_or_else(|_| runtime.domain_error("dyadic_product"))
}

/// Apply `verb` to `value` over the vector.
fn adverb_over(runtime: &Runtime, verb_index: usize, value: Value) -> Value {
    match value {
        Value::Atom(_) => value,
        Value::Vector(vector) => vector.iter().fold(0.into(), |result, integer| {
            apply_dyadic_verb(runtime, verb_index, result, integer.into())
        }),
        Value::Error => Value::Error,
    }
}

/// Apply `verb` to `value` while scanning the vector.
fn adverb_scan(runtime: &Runtime, verb_index: usize, value: Value) -> Value {
    match value {
        Value::Atom(_) => value,
        Value::Vector(vector) => {
            let mut iter = vector.iter();
            let Some(first) = iter.next() else {
                return vec![].into();
            };

            let mut result = Value::Atom(*first);
            let mut output = Vec::with_capacity(vector.len());
            output.push(*first);

            for integer in iter {
                match apply_dyadic_verb(runtime, verb_index, result.clone(), (*integer).into()) {
                    Value::Atom(value) => {
                        result = Value::Atom(value);
                        output.push(value);
                    }
                    _ => {
                        result = Value::Error;
                        output.push(0);
                    }
                }
            }

            output.into()
        }
        Value::Error => Value::Error,
    }
}

const MONADIC_VERBS: [MonadicVerb; 12] = [
    monadic_not_a_verb,
    monadic_not_implemented,
    monadic_negate,
    monadic_enumerate,
    monadic_count,
    monadic_enlist,
    monadic_first,
    monadic_not_implemented,
    monadic_not_implemented,
    monadic_not_implemented,
    monadic_reverse,
    monadic_not_implemented,
];

const DYADIC_VERBS: [DyadicVerb; 12] = [
    dyadic_not_a_verb,
    dyadic_add,
    dyadic_subtract,
    dyadic_modulo,
    dyadic_take,
    dyadic_concatenate,
    dyadic_index_at,
    dyadic_equal,
    dyadic_not_equal,
    dyadic_and,
    dyadic_or,
    dyadic_product,
];

const ADVERBS: [Adverb; 3] = [|_runtime, _, value| value, adverb_over, adverb_scan];

/// Helper function to apply a monadic verb.
pub(crate) fn apply_monadic_verb(runtime: &Runtime, verb_index: usize, value: Value) -> Value {
    let verb = MONADIC_VERBS
        .get(verb_index)
        .copied()
        .unwrap_or(monadic_not_a_verb);
    verb(runtime, value)
}

/// Helper function to apply a dyadic verb.
pub(crate) fn apply_dyadic_verb(
    runtime: &Runtime,
    verb_index: usize,
    left: Value,
    right: Value,
) -> Value {
    let verb = DYADIC_VERBS
        .get(verb_index)
        .copied()
        .unwrap_or(dyadic_not_a_verb);
    verb(runtime, left, right)
}

/// Helper function to apply an adverb.
pub(crate) fn apply_adverb(
    runtime: &Runtime,
    adverb_index: usize,
    verb_index: usize,
    value: Value,
) -> Value {
    let adverb = ADVERBS
        .get(adverb_index)
        .copied()
        .unwrap_or(|_, _, value| value);
    adverb(runtime, verb_index, value)
}
