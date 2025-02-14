use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, GenericArgument, Ident, PathArguments, Type,
};

#[proc_macro_derive(ExpressionType)]
pub fn derive_expression_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // Extract field names for constructor
    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => panic!("ExpressionType can only be derived for structs"),
    };

    // Get fields and their types
    let field_info: Vec<(Ident, Type)> = match fields {
        Fields::Named(fields) => fields
            .named
            .iter()
            .map(|f| {
                let name = f.ident.clone().unwrap();
                let ty = f.ty.clone();
                (name, ty)
            })
            .collect::<Vec<_>>(),
        _ => panic!("ExpressionType requires named fields"),
    };
    let field_names: Vec<_> = field_info.iter().map(|(name, _)| name).collect();

    let deboxed_fields: Vec<_> = field_info
        .iter()
        .map(|(name, ty)| {
            let (ty, did_debox) = deboxed(ty);
            (name, ty, did_debox)
        })
        .collect();

    let deboxed_types: Vec<_> = deboxed_fields
        .iter()
        .map(|(_, ty, _)| {
            quote! { #ty }
        })
        .collect();

    let field_assigns: Vec<_> = deboxed_fields
        .iter()
        .map(|(name, _ty, must_rebox)| {
            if *must_rebox {
                quote! { #name: Box::new(#name) }
            } else {
                quote! { #name }
            }
        })
        .collect();

    let visitor_name = format_ident!("visit_{}", name.to_string().to_lowercase());
    let borrowing_visitor_name = format_ident!("borrow_{}", name.to_string().to_lowercase());

    let expanded = quote! {
        impl #name {
            pub fn expr(#(#field_names: #deboxed_types),*) -> Expr {
                Expr::#name(Self::new(#(#field_names),*))
            }

            pub fn new(#(#field_names: #deboxed_types),*) -> Self {
                Self {
                    #(#field_assigns),*
                }
            }

            fn accept<T>(self, mut visitor: impl Visitor<T>) -> T {
                visitor.#visitor_name(self)
            }

            fn accept_borrowed<T>(&self, visitor: impl BorrowingVisitor<T>) -> T {
                visitor.#borrowing_visitor_name(self)
            }
        }
    };

    TokenStream::from(expanded)
}

fn deboxed(ty: &Type) -> (&Type, bool) {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if let PathArguments::AngleBracketed(ref args) = segment.arguments {
                if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                    return (inner_ty, true);
                }
            }
        }
    }
    (ty, false)
}
