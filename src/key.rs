use std::fmt::{self, Display};

#[derive(Clone)]
pub enum RepresentKey {
    Int(i128),
    Usize(usize),
    Float(f64),
    Static(&'static str),
    Owned(String),
}

impl From<i128> for RepresentKey {
    fn from(value: i128) -> Self {
        Self::Int(value)
    }
}

impl From<usize> for RepresentKey {
    fn from(value: usize) -> Self {
        Self::Usize(value)
    }
}

impl From<f64> for RepresentKey {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<&'static str> for RepresentKey {
    fn from(value: &'static str) -> Self {
        Self::Static(value)
    }
}

impl From<String> for RepresentKey {
    fn from(value: String) -> Self {
        Self::Owned(value)
    }
}

impl Display for RepresentKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RepresentKey::Int(val) => Display::fmt(val, f),
            RepresentKey::Usize(val) => Display::fmt(val, f),
            RepresentKey::Float(val) => Display::fmt(val, f),
            RepresentKey::Static(val) => Display::fmt(val, f),
            RepresentKey::Owned(val) => Display::fmt(val, f),
        }
    }
}
