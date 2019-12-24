extern crate quote;
extern crate proc_macro;
#[macro_use]
extern crate syn;

use proc_macro::TokenStream;
use quote::Tokens;

use syn::DeriveInput;
use syn::Meta::{List, NameValue};
use syn::NestedMeta::{Literal, Meta};
use syn::Data::*;

use syn::{Fields, Ident};

#[proc_macro_derive(PostgresMapper, attributes(pg_mapper))]
pub fn postgres_mapper(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    impl_derive(&ast).parse().expect("Error parsing postgres mapper tokens")
}

fn impl_derive(ast: &DeriveInput) -> Tokens {
    let mut tokens = Tokens::new();

    let fields: &Fields = match ast.data {
        Struct(ref s) => { &s.fields },
        Enum(ref u) => { panic!("Enums can not be mapped") },
        Union(ref u) => { panic!("Unions can not be mapped") },
    };

    impl_tokio_from_row(&mut tokens, &ast.ident, &fields);
    impl_tokio_from_borrowed_row(&mut tokens, &ast.ident, &fields);
    impl_tokio_postgres_mapper(&mut tokens, &ast.ident, &fields);

    tokens
}

fn impl_tokio_from_row(t: &mut Tokens, struct_ident: &Ident, fields: &Fields) {
    t.append(format!("impl From<::tokio_postgres::row::Row> for {struct_name} {{
                          fn from(row: ::tokio_postgres::row::Row) -> Self {{
                              Self {{", struct_name=struct_ident));

    for field in fields {
        let ident = field.ident.clone().expect("Expected structfield identifier");

        t.append(format!("{0}: row.get(\"{0}\"),", ident));
    }

    t.append("}}}");
}

fn impl_tokio_from_borrowed_row(t: &mut Tokens, struct_ident: &Ident, fields: &Fields) {
    t.append(format!("impl<'a> From<&'a ::tokio_postgres::row::Row> for {struct_name} {{
                          fn from(row: &'a ::tokio_postgres::row::Row) -> Self {{
                              Self {{", struct_name=struct_ident));

    for field in fields {
        let ident = field.ident.clone().expect("Expected structfield identifier");

        t.append(format!("{0}: row.get(\"{0}\"),", ident));
    }

    t.append("}}}");
}


fn impl_tokio_postgres_mapper(t: &mut Tokens, struct_ident: &Ident, fields: &Fields) {
    t.append(format!("impl ::postgres_mapper::FromTokioPostgresRow for {struct_name} {{
                          fn from_tokio_postgres_row(row: ::tokio_postgres::row::Row) -> Result<Self, ::postgres_mapper::Error> {{
                              Ok(Self {{", struct_name=struct_ident));

    for field in fields {
        let ident = field.ident.clone().expect("Expected structfield identifier");

        t.append(format!("{0}: row.try_get(\"{0}\")?.ok_or_else(|| ::postgres_mapper::Error::ColumnNotFound)?,", ident));
    }

    t.append("})}");

    t.append("fn from_tokio_postgres_row_ref(row: &::tokio_postgres::row::Row) -> Result<Self, ::postgres_mapper::Error> {
                  Ok(Self {");

    for field in fields {
        let ident = field.ident.clone().expect("Expected structfield identifier");

        t.append(format!("{0}: row.try_get(\"{0}\")?.ok_or_else(|| ::postgres_mapper::Error::ColumnNotFound)?,", ident));
    }

    t.append("})}}");
}