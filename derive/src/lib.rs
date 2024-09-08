use proc_macro2::TokenStream;
//use proc_macro::TokenStream;
//use syn::{parse_macro_input, DeriveInput};
use quote::quote;
use syn::{Attribute, Expr, Fields, Ident, Lit, Meta, MetaNameValue, NestedMeta, Pat, Type};
use synstructure::decl_derive;

#[derive(Default)]
struct IdentCounter(usize);

impl IdentCounter {
    fn next_key(&mut self, ident: &Option<Ident>) -> TokenStream {
        match ident {
            Some(ident) => {
                quote!(stringify!(#ident))
            }
            None => {
                let index = self.0;
                self.0 += 1;
                quote!(#index)
            }
        }
    }
}

fn visit_derive(s: synstructure::Structure) -> TokenStream {
    let mut body = TokenStream::default();
    for v in s.variants() {
        let mut counter = IdentCounter::default();
        let is_enum = v.prefix.is_some();
        let variant_ident = v.ast().ident;

        let pat = v.pat();
        let keys = v
            .bindings()
            .iter()
            .map(|bi| counter.next_key(&bi.ast().ident));
        let bi = v.bindings().iter();
        let per_variant = quote!(
            {
                #(visitor.visit_keyed(#keys, #bi)?;)*
                //Ok::<_, <V as Visitor>::Error>(())
                Ok(())
            }
        );
        if is_enum {
            body.extend(quote! {
                #pat => {
                    visitor.with_key(stringify!(#variant_ident).into(), |visitor| #per_variant)
                }
            });
        } else {
            body.extend(quote! {
                #pat => {
                    #per_variant
                }
            });
        }
    }

    let mut bounds = vec![];
    for variant in s.variants() {
        for binding in variant.bindings() {
            let bounded_ty = binding.ast().ty.clone();
            if !bounds.iter().any(|ty| ty == &bounded_ty) {
                bounds.push(bounded_ty);
            }
        }
    }

    s.gen_impl(quote! {
        use represent::{VisitWith, VisitType, Visitor};
        gen impl<V> VisitWith<V> for @Self
            where V: Visitor #(+ VisitType<#bounds>)*
        {
            fn visit_with(&self, visitor: &mut V) -> Result<(), <V as Visitor>::Error> {
                match self { #body }
            }
        }
    })
}
decl_derive!([VisitWith, attributes(alt)] => visit_derive);

fn make_with_derive(s: synstructure::Structure) -> TokenStream {
    let alt = get_alt(&s.ast().attrs);
    if !alt.is_empty() {
        return make_with_derive_alt(s, alt);
    }
    assert_eq!(s.variants().len(), 1);

    let structure = &s.variants()[0];

    let mut bounds = vec![];
    let mut idents = vec![];
    for binding in structure.bindings() {
        idents.push(&binding.ast().ident);
        let bounded_ty = &binding.ast().ty;
        if !bounds.iter().any(|ty| ty == &bounded_ty) {
            bounds.push(bounded_ty);
        }
    }

    let mut counter = IdentCounter::default();
    let keys = idents.iter().map(|ident| counter.next_key(ident));

    let ok = if is_tuple(structure.ast().fields) {
        quote!(
            Ok(Self (
                #(#idents
                    maker.make_keyed(#keys)?,
                )*
            ))
        )
    } else {
        quote!(
            Ok(Self {
                #(#idents:
                    maker.make_keyed(#keys)?,
                )*
            })
        )
    };

    s.gen_impl(quote! {
        use represent::{MakeWith, MakeType, Maker};
        gen impl<M> MakeWith<M> for @Self
            where M: Maker #(+ MakeType<#bounds>)*
        {
            fn make_with(maker: &mut M) -> Result<Self, <M as Maker>::Error> {
                #ok
            }
        }
    })
}
decl_derive!([MakeWith, attributes(alt)] => make_with_derive);

struct StructAlt {
    ty: Type,
    default: Option<Expr>,
    err: Option<Type>,
    no_bounds: bool,
}
impl StructAlt {
    fn new(vec: Vec<NestedMeta>) -> Self {
        let mut ty = None;
        let mut default = None;
        let mut err = None;
        let mut no_bounds = false;
        for item in vec {
            match item {
                NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                    path,
                    lit: Lit::Str(value),
                    ..
                })) => {
                    let Some(ident) = path.get_ident() else {
                        panic!("alt path {:?} is unsupported", path);
                    };
                    if ident == "ty" {
                        ty = Some(value.parse().unwrap());
                    } else if ident == "default" {
                        default = Some(value.parse().unwrap());
                    } else if ident == "err" {
                        err = Some(value.parse().unwrap());
                    } else if ident == "bounds" {
                        no_bounds = &value.value() == "false";
                    } else {
                        panic!("alt ident {:?} is unsupported", ident);
                    }
                }
                other => panic!("{:?} alt attr is unsupported", other),
            }
        }
        Self {
            ty: ty.unwrap(),
            default,
            err,
            no_bounds,
        }
    }
}

fn get_alt(attrs: &[Attribute]) -> Vec<NestedMeta> {
    attrs
        .iter()
        .filter(|attr| {
            if let Some(ident) = attr.path.get_ident() {
                ident == "alt"
            } else {
                false
            }
        })
        .map(|attr| attr.parse_meta().unwrap())
        .flat_map(|meta| match meta {
            Meta::List(list) => list.nested,
            _ => unimplemented!(),
        })
        .collect()
}

fn get_alt_pattern(attrs: &[Attribute]) -> Option<Pat> {
    match get_alt(attrs).first()? {
        NestedMeta::Lit(Lit::Str(lit)) => lit.parse().ok(),
        _ => None,
    }
}

fn is_tuple(fields: &Fields) -> bool {
    matches!(fields, Fields::Unnamed(..))
}

fn make_with_derive_alt(st: synstructure::Structure, struct_alt: Vec<NestedMeta>) -> TokenStream {
    let StructAlt {
        ty,
        default,
        err,
        no_bounds,
    } = StructAlt::new(struct_alt);
    let mut bounds = vec![];
    let mut variants = vec![];
    struct Alt<'a> {
        name: &'a Ident,
        cond: Pat,
        idents: Vec<&'a Option<Ident>>,
        tuple: bool,
    }
    for variant in st.variants() {
        let Some(cond) = get_alt_pattern(variant.ast().attrs) else {
            continue;
        };

        let mut idents = vec![];
        for binding in variant.bindings() {
            idents.push(&binding.ast().ident);
            let bounded_ty = &binding.ast().ty;
            if !no_bounds && !bounds.iter().any(|ty| ty == &bounded_ty) {
                bounds.push(bounded_ty);
            }
        }
        variants.push(Alt {
            name: variant.ast().ident,
            cond,
            idents,
            tuple: is_tuple(variant.ast().fields),
        });
    }

    let conds = variants.iter().map(|alt| &alt.cond);
    let expr = variants.iter().map(|alt| {
        let idents = alt.idents.as_slice();
        let name = alt.name;
        let mut counter = IdentCounter::default();
        let keys = idents.iter().map(|ident| counter.next_key(ident));
        let expr = if alt.idents.is_empty() {
            quote!(Ok(Self::#name))
        } else if alt.tuple {
            let args = quote!(
                #(#idents
                    maker.make_keyed(#keys)?,
                )*
            );
            quote!(
                Ok(Self::#name(#args))
            )
        } else {
            let args = quote!(
                #(#idents:
                    maker.make_keyed(#keys)?,
                )*
            );
            quote!(
                Ok(Self::#name{#args})
            )
        };
        quote!({
            maker.with_key(stringify!(#name).into(), |maker| {#expr})?
        })
    });
    let default = if let Some(default) = default {
        quote!(_ => #default)
    } else {
        quote!()
    };
    let err = if let Some(err) = err {
        quote!(#err: Into<M::Error>)
    } else {
        quote!()
    };
    st.gen_impl(quote! {
        use represent::{MakeWith, MakeType, Maker};
        gen impl<M> MakeWith<M> for @Self
            where M: Maker + MakeType<#ty> #(+ MakeType<#bounds>)*,
            #err
        {
            fn make_with(maker: &mut M) -> Result<Self, <M as Maker>::Error> {
                let alt: #ty = MakeType::make_type(maker)?;
                match alt {
                    #(
                        #conds => Ok(#expr),
                    )*
                    #default
                }
            }
        }
    })
}

fn analyze_derive(s: synstructure::Structure) -> TokenStream {
    let structure = &s.variants()[0];

    let mut bounds = vec![];
    let mut types = vec![];
    let mut idents = vec![];
    for binding in structure.bindings() {
        types.push(&binding.ast().ty);
        idents.push(&binding.ast().ident);
        let bounded_ty = &binding.ast().ty;
        if !bounds.iter().any(|ty| *ty == bounded_ty) {
            bounds.push(bounded_ty);
        }
    }

    s.gen_impl(quote! {
        use represent::{AnalyzeWith, AnalyzeType, TypeAnalyzer, TypeSize, sum_sizes};
        gen impl<A> AnalyzeWith<A> for @Self
        where
            A: TypeAnalyzer #(+ AnalyzeType<#bounds>)*
        {
            const CONST_SIZE: TypeSize = sum_sizes([
                    #(<A as AnalyzeType<#types>>::TYPE_CONST_SIZE),*
            ]);

            fn fixed_size(analyzer: &A) -> usize {
                let arr = [
                    0usize,
                    #(<A as AnalyzeType<#types>>::type_fixed_size(analyzer)),*
                ];
                std::array::IntoIter::new(arr).sum()
            }

            fn dynamic_size(&self, analyzer: &A) -> usize {
                let arr = [
                    0usize,
                    #(analyzer.type_dynamic_size(&self.#idents)),*
                ];
                std::array::IntoIter::new(arr).sum()
            }
        }
    })
}
decl_derive!([AnalyzeWith] => analyze_derive);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[ignore]
    #[allow(clippy::all)]
    fn test() {
        use synstructure::test_derive;
        test_derive! {
            visit_derive {
                struct A {
                    a: u8,
                    b: u8,
                    c: u16,
                }
            }
            expands to {
                #[allow(non_upper_case_globals)]
                const _DERIVE_Visit_V_FOR_A: () = {
                    use represent::{VisitWith, VisitType, Visitor};
                    impl<V> VisitWith<V> for A
                    where
                        V: Visitor + VisitType<u8> + VisitType<u16>
                    {
                        fn visit_with(&self, visitor: &mut V) -> Result<(), <V as Visitor>::Error> {
                            match self {
                                A {
                                    a: ref __binding_0,
                                    b: ref __binding_1,
                                    c: ref __binding_2,
                                } => {
                                    {
                                        VisitType::visit(visitor, __binding_0)?;
                                    }
                                    {
                                        VisitType::visit(visitor, __binding_1)?;
                                    }
                                    {
                                        VisitType::visit(visitor, __binding_2)?;
                                    }
                                }
                            }
                            Ok(())
                        }
                    }
                };

            }
        }
    }
}
