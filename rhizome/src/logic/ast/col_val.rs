use std::fmt::{self, Display};

use crate::value::Val;

use super::Var;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ColVal {
    Lit(Val),
    Binding(Var),
}

impl From<Val> for ColVal {
    fn from(value: Val) -> Self {
        Self::Lit(value)
    }
}

impl From<Var> for ColVal {
    fn from(value: Var) -> Self {
        Self::Binding(value)
    }
}

impl From<&Var> for ColVal {
    fn from(value: &Var) -> Self {
        Self::Binding(*value)
    }
}

impl Display for ColVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColVal::Lit(inner) => Display::fmt(&inner, f),
            ColVal::Binding(inner) => Display::fmt(&inner, f),
        }
    }
}