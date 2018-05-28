extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use std::iter;

use proc_macro::TokenStream;

#[proc_macro_derive(Visit, attributes(visit))]
pub fn visit_derive(input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();

    // Parse the string representation
    let ast = syn::parse_macro_input(&s).expect("Unable to parse input");

    // Build the impl
    let gen = impl_visit(&ast);

    // Return the generated impl
    gen.parse().expect("Unable to generate")
}

fn impl_visit(ast: &syn::MacroInput) -> quote::Tokens {
    let name = &ast.ident;
    let method_name_base = camelcase_to_snake_case(&name.to_string());
    let method_name: quote::Ident = format!("visit{}", method_name_base).into();
    let exit_method_name: quote::Ident = format!("exit{}", method_name_base).into();

    let visit_fields = impl_visit_fields(ast, IsMut(false));
    let visit_fields_mut = impl_visit_fields(ast, IsMut(true));

    quote! {
        impl ::visit::Visit for #name {
            fn visit<'ast, V>(&'ast self, v: &mut V)
            where
                V: ::visit::Visitor<'ast>,
            {
                if ::visit::Control::Continue == v.#method_name(self) {
                    #visit_fields;
                }
                v.#exit_method_name(self);
            }

            fn visit_mut<V>(&mut self, v: &mut V)
            where
                V: ::visit::VisitorMut,
            {
                if ::visit::Control::Continue == v.#method_name(self) {
                    #visit_fields_mut;
                }
                v.#exit_method_name(self);
            }
        }
    }
}

struct IsMut(bool);

fn impl_visit_fields(ast: &syn::MacroInput, IsMut(is_mut): IsMut) -> quote::Tokens {
    use syn::{Body, VariantData};

    let method = if is_mut {
        "::visit::Visit::visit_mut"
    } else {
        "::visit::Visit::visit"
    };
    let method = iter::repeat(syn::Ident::from(method));

    match ast.body {
        Body::Enum(ref e) => {
            let enum_name = iter::repeat(&ast.ident);
            let variant_names = e.iter().map(|variant| &variant.ident);

            if is_mut {
                quote! {
                    match *self {
                        #(#enum_name::#variant_names(ref mut x) => #method(x, v),)*
                    }
                }
            } else {
                quote! {
                    match *self {
                        #(#enum_name::#variant_names(ref x) => #method(x, v),)*
                    }
                }
            }
        }
        Body::Struct(VariantData::Struct(ref fields))
        | Body::Struct(VariantData::Tuple(ref fields)) => {
            let field_names = fields
                .iter()
                .enumerate()
                .filter(|&(_, ref f)| !is_ignore_field(f))
                .map(|(i, f)| f.ident.clone().unwrap_or_else(|| i.into()));

            if is_mut {
                quote! {
                    #(#method(&mut self.#field_names, v);)*
                }
            } else {
                quote! {
                    #(#method(&self.#field_names, v);)*
                }
            }
        }
        Body::Struct(VariantData::Unit) => quote!{},
    }
}

fn is_ignore_field(field: &syn::Field) -> bool {
    use syn::MetaItem;

    let attr_name: syn::Ident = "visit".into();

    field.attrs.iter().any(|attr| match attr.value {
        MetaItem::List(ref name, ref children) => {
            name == &attr_name && children.iter().any(ignore_field_inner)
        }
        _ => false,
    })
}

fn ignore_field_inner(item: &syn::NestedMetaItem) -> bool {
    use syn::{MetaItem, NestedMetaItem};

    let ignore_value: syn::Ident = "ignore".into();

    match *item {
        NestedMetaItem::MetaItem(MetaItem::Word(ref i)) => i == &ignore_value,
        _ => false,
    }
}

fn camelcase_to_snake_case(camelcase: &str) -> String {
    let mut s = String::new();

    for c in camelcase.chars() {
        if c.is_lowercase() {
            s.push(c);
        } else {
            s.push('_');
            s.extend(c.to_lowercase());
        }
    }

    s
}
