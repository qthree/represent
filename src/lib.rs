mod key;
use std::marker::PhantomData;

pub use key::RepresentKey;
#[cfg(feature = "derive")]
pub use represent_derive::{AnalyzeWith, MakeWith, VisitWith};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
pub trait Visitor {
    type Error;
    fn with_key(
        &mut self,
        _key: RepresentKey,
        func: impl FnOnce(&mut Self) -> Result<(), Self::Error>,
    ) -> Result<(), Self::Error> {
        func(self)
    }
    fn visit_keyed<T>(
        &mut self,
        key: impl Into<RepresentKey>,
        target: &T,
    ) -> Result<(), <Self as Visitor>::Error>
    where
        Self: VisitType<T>,
    {
        self.with_key(key.into(), |visitor| visitor.visit(target))
    }
}

pub trait VisitType<T>: Visitor {
    fn visit(&mut self, target: &T) -> Result<(), <Self as Visitor>::Error>;
}

pub trait VisitWith<V: Visitor> {
    fn visit_with(&self, visitor: &mut V) -> Result<(), <V as Visitor>::Error>;
}

pub trait Maker {
    type Error;
    fn make<T>(&mut self) -> Result<T, Self::Error>
    where
        Self: MakeType<T>,
    {
        self.make_type()
    }
    fn with_key<R>(
        &mut self,
        _key: RepresentKey,
        func: impl FnOnce(&mut Self) -> Result<R, Self::Error>,
    ) -> Result<R, Self::Error> {
        func(self)
    }
    fn make_keyed<T>(&mut self, key: impl Into<RepresentKey>) -> Result<T, Self::Error>
    where
        Self: MakeType<T>,
    {
        self.with_key(key.into(), |maker| maker.make_type())
    }
}
pub trait MakeType<T: Sized>: Maker {
    fn make_type(&mut self) -> Result<T, <Self as Maker>::Error>;
}
pub trait MakeWith<M: Maker>: Sized {
    fn make_with(maker: &mut M) -> Result<Self, <M as Maker>::Error>;
}
pub trait TypeAnalyzer {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeSize {
    Const(usize),
    Fixed,
    Dynamic,
}

impl TypeSize {
    pub const fn expect_const(self) -> usize {
        match self {
            TypeSize::Const(size) => size,
            _ => panic!("Expected const size"),
        }
    }

    const fn is_const_or_fixed(self) -> bool {
        matches!(self, TypeSize::Const(_) | TypeSize::Fixed)
    }
}

pub struct AssertConstOrFixed<A: TypeAnalyzer, S: AnalyzeWith<A>>(PhantomData<A>, PhantomData<S>);
impl<A: TypeAnalyzer, S: AnalyzeWith<A>> AssertConstOrFixed<A, S> {
    const ASSERT_CONST_OR_FIXED: () = assert!(S::CONST_SIZE.is_const_or_fixed());

    #[expect(path_statements)]
    pub fn assert() {
        Self::ASSERT_CONST_OR_FIXED;
    }
}

pub trait AnalyzeType<T>: TypeAnalyzer {
    const TYPE_CONST_SIZE: TypeSize;

    fn type_fixed_size(&self) -> usize {
        match Self::TYPE_CONST_SIZE {
            TypeSize::Const(size) => size,
            TypeSize::Fixed => unimplemented!("Fixed size should be implemented manually!"),
            TypeSize::Dynamic => {
                panic!("fixed_size() should only be called on type with TypeSize::Fixed")
            }
        }
    }

    fn type_dynamic_size(&self, _target: &T) -> usize {
        match Self::TYPE_CONST_SIZE {
            TypeSize::Const(size) => size,
            TypeSize::Fixed => self.type_fixed_size(),
            TypeSize::Dynamic => unimplemented!("dynamic_size() should be implemented manually!"),
        }
    }
}

pub trait AnalyzeWith<A: TypeAnalyzer> {
    const CONST_SIZE: TypeSize;

    fn fixed_size(_analyzer: &A) -> usize {
        match Self::CONST_SIZE {
            TypeSize::Const(size) => size,
            TypeSize::Fixed => unimplemented!("Fixed size should be implemented manually!"),
            TypeSize::Dynamic => {
                panic!("fixed_size() should only be called on type with TypeSize::Fixed")
            }
        }
    }

    fn dynamic_size(&self, analyzer: &A) -> usize {
        match Self::CONST_SIZE {
            TypeSize::Const(size) => size,
            TypeSize::Fixed => Self::fixed_size(analyzer),
            TypeSize::Dynamic => unimplemented!("dynamic_size() should be implemented manually!"),
        }
    }
}

pub const fn sum_sizes<const L: usize>(sizes: [TypeSize; L]) -> TypeSize {
    let mut i = 0;
    let mut size = TypeSize::Const(0);
    while i < L {
        match (size, sizes[i]) {
            (TypeSize::Const(sum), TypeSize::Const(add)) => {
                size = TypeSize::Const(sum + add);
            }
            (TypeSize::Const(_), TypeSize::Fixed) => {
                size = TypeSize::Fixed;
            }
            (_, TypeSize::Dynamic) => {
                return TypeSize::Dynamic;
            }
            _ => {}
        }
        i += 1;
    }
    size
}
