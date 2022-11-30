use proc_macro::{TokenStream};
use convert_case::{Case, Casing};
use syn::{Error, Ident, ItemStruct, parse_macro_input};
use syn::ext::IdentExt;
use quote::quote;
use syn::Lit::Str;
use syn::Meta::{List, NameValue, Path};
use syn::NestedMeta::Meta;
use syn::spanned::Spanned;
use crate::column::Column;
use crate::symbol::{COLUMN, COLUMN_NAME, COMMIT_TIMESTAMP};
use crate::util::wrap_in_dummy_mod;

pub(crate) fn generate_table_methods(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStruct);
    let struct_name = item.ident;

    let mut to_kinds_fields = Vec::with_capacity(item.fields.len());
    let mut get_types_fields = Vec::with_capacity(item.fields.len());
    let mut try_from_struct_fields = Vec::with_capacity(item.fields.len());
    for field in &item.fields {
        let field_var = field.ident.as_ref().unwrap();
        let column = Column::from_ast(&field);
        let column_name = column.column_name.unwrap_or_else(|| field_var.unraw().to_string().to_case(Case::Title));
        let ty = &field.ty;
        let mut get_field_type  =  quote! { #ty };
        let mut to_kind_field_type = quote! { self.#field_var };
        if column.commit_timestamp {
            get_field_type = quote! { CommitTimestamp };
            to_kind_field_type = quote! { CommitTimestamp::new() };
        }
        to_kinds_fields.push(quote! {
            (stringify!(#column_name), #to_kind_field_type.to_kind())
        });
        get_types_fields.push(quote! {
            (stringify!(#column_name), #get_field_type::get_type())
        });
        try_from_struct_fields.push(quote! {
            #field_var: s.column_by_name(#column_name)?
        });
    }

    let gen = quote! {

        impl ToStruct for #struct_name  {

            fn to_kinds(&self) -> Kinds {
                vec![
                    #(
                        #to_kinds_fields,
                    )*
                ]
            }
            fn get_types() -> Types {
                vec![
                    #(
                        #get_types_fields,
                    )*
                ]
            }
        }

        impl TryFromStruct for #struct_name {
            fn try_from_struct(s: Struct<'_>) -> Result<Self, RowError> {
                Ok(#struct_name {
                    #(
                        #try_from_struct_fields,
                    )*
                })
            }
        }

        impl std::convert::TryFrom<Row> for #struct_name {
            type Error = RowError;
            fn try_from(s: Row) -> Result<Self, RowError> {
                Ok(#struct_name {
                    #(
                        #try_from_struct_fields,
                    )*
                })
            }
        }

    };

    wrap_in_dummy_mod(gen)
}
