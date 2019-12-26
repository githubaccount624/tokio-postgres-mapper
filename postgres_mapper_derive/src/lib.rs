extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;

use syn::{
    Data, DataStruct, DeriveInput, GenericParam, Ident, ImplGenerics, Item, Lifetime, LifetimeDef,
    TypeGenerics, WhereClause,
};

#[proc_macro_derive(PostgresMapper)]
pub fn postgres_mapper(input: TokenStream) -> TokenStream {
    let mut ast: DeriveInput = syn::parse(input).expect("Couldn't parse item");

    impl_derive(&mut ast)
}

fn impl_derive(ast: &mut DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();

    let s = match ast.data {
        Data::Struct(ref s) => s,
        _ => panic!("Enums or Unions can not be mapped"),
    };

    let from_row = impl_tokio_from_row(s, name, &impl_generics, &ty_generics, &where_clause);

    // let tokio_postgres_mapper =
    //     impl_tokio_postgres_mapper(s, name, impl_generics, ty_generics, where_clause);

    let mut g = ast.generics.clone();

    g.params.push(GenericParam::Lifetime(LifetimeDef::new(
        syn::parse_str::<Lifetime>("'DERIVE").unwrap(),
    )));
    let (impl_generics, _, _) = { g.split_for_impl() };

    let from_row_borrowed =
        impl_tokio_from_row_ref(s, name, &impl_generics, ty_generics, where_clause);

    let tokens = quote! {
        #from_row
        #from_row_borrowed
        // #tokio_postgres_mapper
    };

    tokens.into()
}

fn impl_tokio_from_row(
    s: &DataStruct,
    name: &Ident,
    impl_generics: &ImplGenerics,
    ty_generics: &TypeGenerics,
    where_clause: &Option<&WhereClause>,
) -> Item {
    let fields = s.fields.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();

        let row_expr = format!(r##"{}"##, ident);
        quote! {
            #ident:row.get(#row_expr)
        }
    });

    let tokens = quote! {
        impl #impl_generics From<tokio_postgres::row::Row> for #name #ty_generics #where_clause  {
            fn from(row:tokio_postgres::row::Row) -> Self {
                Self {
                    #(#fields),*
                }
            }
        }
    };

    syn::parse_quote!(#tokens)
}

fn impl_tokio_from_row_ref(
    s: &DataStruct,
    name: &Ident,
    impl_generics: &ImplGenerics,
    ty_generics: &TypeGenerics,
    where_clause: &Option<&WhereClause>,
) -> Item {
    let fields = s.fields.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();

        let row_expr = format!(r##"{}"##, ident);
        quote! {
            #ident:row.get(#row_expr)
        }
    });

    let tokens = quote! {
        impl #impl_generics From<&'DERIVE tokio_postgres::row::Row> for #name #ty_generics #where_clause {
            fn from(row:&'DERIVE tokio_postgres::row::Row) -> Self {
                Self {
                    #(#fields),*
                }
            }
        }
    };

    syn::parse_quote!(#tokens)
}

// fn impl_tokio_postgres_mapper(
//     s: &DataStruct,
//     name: &Ident,
//     impl_generics: &ImplGenerics,
//     ty_generics: &TypeGenerics,
//     where_clause: &Option<&WhereClause>,
// ) -> Item {
//     let fields = s.fields.iter().map(|field| {
//         let ident = field.ident.as_ref().unwrap();

//         let row_expr = format!(r##"{}"##, ident);
//         quote! {
//             #ident:row.try_get(#row_expr)?.ok_or_else(|| tokio_postgres_mapper::Error::ColumnNotFound)?
//         }
//     });

//     let fields_copy = fields.clone();

//     let tokens = quote! {
//         impl #impl_generics tokio_postgres_mapper::FromTokioPostgresRow for #name #ty_generics #where_clause {
//             fn from_tokio_postgres_row(row: tokio_postgres::row::Row) -> Result<Self, tokio_postgres_mapper::Error> {
//                 Ok(Self {
//                     #(#fields),*
//                 })
//             }

//             fn from_tokio_postgres_row_ref(row: &tokio_postgres::row::Row) -> Result<Self, tokio_postgres_mapper::Error> {
//                 Ok(Self {
//                     #(#fields_copy),*
//                 })
//             }
//         }
//     };

//     syn::parse_quote!(#tokens)
// }
