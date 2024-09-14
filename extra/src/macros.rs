#[macro_export]
macro_rules! impl_final_const (
    (
        [$($bounds:tt)*] for $TY:ty = $CONST:expr
    ) => {
        impl<D: represent::TypeAnalyzer, $($bounds)*> represent::AnalyzeWith<D> for $TY {
            const CONST_SIZE: represent::TypeSize = represent::TypeSize::Const($CONST);
            fn fixed_size(_analyzer: &D) -> usize {
                unimplemented!()
            }
            fn dynamic_size(&self, _analyzer: &D) -> usize {
                unimplemented!()
            }
        }
    }
);

#[macro_export]
macro_rules! impl_visit_empty {
    ([$($bounds:tt)*] for $typ:ty) => {
        impl<$($bounds)*, VISITOR: represent::Visitor> represent::VisitWith<VISITOR> for $typ {
            fn visit_with(&self, _visitor: &mut VISITOR) -> Result<(), VISITOR::Error> {
                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! impl_make_default {
    ([$($bounds:tt)*] for $typ:ty) => {
        impl<M: represent::Maker, $($bounds)*> represent::MakeWith<M> for $typ {
            fn make_with(_maker: &mut M) -> Result<$typ, M::Error> {
                Ok(Default::default())
            }
        }
    };
}

#[macro_export]
macro_rules! impl_analyzer (
    (
        [] for $TY:ty
    ) => {
        impl represent::TypeAnalyzer for $TY {}
        $crate::impl_analyzer!(#analyze_type [T: represent::AnalyzeWith<Self>] for $TY);
    };
    (
        [$($bounds:tt)+] for $TY:ty
    ) => {
        impl<$($bounds)*> represent::TypeAnalyzer for $TY {}
        $crate::impl_analyzer!(#analyze_type [$($bounds)+, T: represent::AnalyzeWith<Self>] for $TY);
    };
    (
        #analyze_type [$($bounds:tt)+] for $TY:ty
    ) => {
        impl<$($bounds)+> represent::AnalyzeType<T> for $TY {
            const TYPE_CONST_SIZE: represent::TypeSize = T::CONST_SIZE;

            fn type_fixed_size(&self) -> usize {
                T::fixed_size(self)
            }

            fn type_dynamic_size(&self, target: &T) -> usize {
                target.dynamic_size(self)
            }
        }
    }
);
