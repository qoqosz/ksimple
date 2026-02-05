use std::fmt::Display;
use std::ops::Neg;
use std::rc::Rc;

/// A value in the k/simple programming language.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Value {
    Atom(i64),
    Vector(Rc<Vec<i64>>),
    Error,
}

impl Value {
    pub(crate) fn is_error(&self) -> bool {
        matches!(self, Self::Error)
    }

    /// Enlist a value.
    pub(crate) fn enlist(&self) -> Result<Self, ()> {
        match self {
            Self::Atom(integer) => Ok(vec![*integer].into()),
            Self::Vector(_) => Err(()),
            Self::Error => Ok(Self::Error),
        }
    }

    /// Reverse a value.
    pub(crate) fn reverse(&self) -> Result<Self, ()> {
        match self {
            Self::Atom(_) => Err(()),
            Self::Vector(vector) => Ok(vector.iter().rev().cloned().collect::<Vec<_>>().into()),
            Self::Error => Ok(Self::Error),
        }
    }

    /// Apply a dyadic verb to a value.
    pub(crate) fn apply_dyadic_verb(
        &self,
        other: &Self,
        verb: fn(i64, i64) -> i64,
    ) -> Result<Self, ()> {
        match (self, other) {
            (Self::Atom(a), Self::Atom(b)) => Ok(verb(*a, *b).into()),
            (Self::Vector(a), Self::Atom(b)) => {
                Ok(a.iter().map(|x| verb(*x, *b)).collect::<Vec<_>>().into())
            }
            (Self::Atom(_), Self::Vector(_)) => other.apply_dyadic_verb(self, verb),
            (Self::Vector(a), Self::Vector(b)) => {
                if a.len() != b.len() {
                    return Err(());
                }
                Ok(a.iter()
                    .zip(b.iter())
                    .map(|(x, y)| verb(*x, *y))
                    .collect::<Vec<_>>()
                    .into())
            }
            _ => Ok(Self::Error),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Atom(integer) => write!(f, "{}", integer),
            Value::Vector(vector) => {
                for integer in vector.iter() {
                    write!(f, "{} ", integer)?;
                }
                Ok(())
            }
            Value::Error => write!(f, "Error"),
        }
    }
}

impl Neg for Value {
    type Output = Value;

    fn neg(self) -> Self::Output {
        match self {
            Value::Atom(integer) => Value::Atom(integer.wrapping_neg()),
            Value::Vector(vector) => vector
                .iter()
                .map(|integer| integer.wrapping_neg())
                .collect::<Vec<_>>()
                .into(),
            Value::Error => Value::Error,
        }
    }
}

/// Helpers
macro_rules! impl_from_integer {
    ($($t:ty),*) => {
        $(
            impl From<$t> for Value {
                fn from(value: $t) -> Self {
                    Value::Atom(value as i64)
                }
            }

            impl From<&$t> for Value {
                fn from(value: &$t) -> Self {
                    Value::Atom(*value as i64)
                }
            }
        )*
    };
}

impl_from_integer!(i8, i16, i32, i64, isize, u8, u16, u32, usize);

impl From<Vec<i64>> for Value {
    fn from(value: Vec<i64>) -> Self {
        Value::Vector(Rc::new(value))
    }
}
