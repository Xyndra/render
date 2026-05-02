// WARNING: AI GENERATED; UNDER REVIEW

//! Proc macro for automatically adding layout fields and implementing Layoutable trait.
//!
//! This crate provides the `#[layoutable]` attribute macro that can be applied to structs
//! to automatically add width, height, x, y, and children fields, and implement the
//! Layoutable trait from render_layout.

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Data, DeriveInput, FieldMutability, Fields, Ident, Type, Visibility, parse_macro_input,
    token::Pub,
};

/// Attribute macro that adds layout fields and implements Layoutable trait.
///
/// This macro can be applied to structs to automatically add:
/// - `width: u32`
/// - `height: u32`
/// - `x: u32`
/// - `y: u32`
/// - `children: Vec<Box<dyn render_layout::Layoutable>>`
///
/// If any of these fields already exist in the struct, they will not be duplicated.
/// The macro also implements the `Layoutable` trait with getters and setters for
/// all these fields.
///
/// # Parameters
///
/// - `custom_default`: Optional parameter that tells the macro not to generate a
///   `Default` implementation for the struct. Use this when your struct has a
///   custom `Default` implementation. The macro will still generate the `new()`
///   method which uses `Self::default()`.
#[proc_macro_attribute]
pub fn layoutable(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input struct
    let mut input = parse_macro_input!(item as DeriveInput);

    // Check if custom_default parameter is present
    let has_custom_default = {
        let attr_str = attr.to_string();
        attr_str.contains("custom_default")
    };

    // Check if it's a struct
    let struct_data = match &mut input.data {
        Data::Struct(data) => data,
        _ => {
            return syn::Error::new_spanned(&input, "layoutable can only be applied to structs")
                .to_compile_error()
                .into();
        }
    };

    // Get the fields of the struct
    let fields = match &mut struct_data.fields {
        Fields::Named(fields) => &mut fields.named,
        _ => {
            return syn::Error::new_spanned(
                &struct_data.fields,
                "layoutable can only be applied to structs with named fields",
            )
            .to_compile_error()
            .into();
        }
    };

    // Collect existing field names to avoid duplicates
    let existing_fields: Vec<String> = fields
        .iter()
        .filter_map(|f| f.ident.as_ref().map(|id| id.to_string()))
        .collect();

    // Add missing fields
    let missing_fields = get_missing_fields(&existing_fields);

    for field in missing_fields {
        fields.push(field);
    }

    // Generate Default implementation if not present and custom_default not specified
    let default_impl = if has_custom_default {
        // User specified custom_default, so they'll provide their own Default impl
        quote! {}
    } else {
        generate_default_impl(&input.ident, &input.attrs, fields)
    };

    // Generate the Layoutable trait implementation
    let struct_name = &input.ident;
    let trait_impl = generate_layoutable_impl(struct_name);

    // Combine the modified struct, Default impl, and trait implementation
    let output = quote! {
        #input

        #default_impl

        #trait_impl
    };

    output.into()
}

/// Determines which fields are missing and need to be added.
fn get_missing_fields(existing_fields: &[String]) -> Vec<syn::Field> {
    let mut fields = Vec::new();

    let required_fields = [
        ("width", "u32"),
        ("height", "u32"),
        ("x", "u32"),
        ("y", "u32"),
        (
            "children",
            "Vec<Box<dyn render_layout::InternalLayoutable>>",
        ),
    ];

    for (field_name, field_type) in required_fields {
        if !existing_fields.contains(&field_name.to_string()) {
            let ty = syn::parse_str::<Type>(field_type)
                .expect("Failed to parse field type {field_type}");

            let field = syn::Field {
                mutability: FieldMutability::None,
                attrs: Vec::new(),
                vis: Visibility::Public(Pub::default()),
                ident: Some(format_ident!("{}", field_name)),
                colon_token: Default::default(),
                ty,
            };

            fields.push(field);
        }
    }

    fields
}

/// Checks if the struct has #[derive(Default)] attribute and generates Default impl if not.
fn generate_default_impl(
    struct_name: &Ident,
    attrs: &[syn::Attribute],
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> proc_macro2::TokenStream {
    // Check if struct already derives Default
    let has_default_derive = attrs.iter().any(|attr| {
        if let syn::Meta::List(meta_list) = &attr.meta
            && meta_list.path.is_ident("derive")
        {
            let tokens = meta_list.tokens.to_string();
            return tokens.contains("Default");
        }
        false
    });

    if has_default_derive {
        // Already has Default via derive, no need to generate
        return quote! {};
    }

    // Check if there's already an explicit Default impl by looking for
    // "impl Default for #struct_name" in the token stream
    // We can't easily detect this, so we'll generate one anyway
    // If there's a conflict, the compiler will error and user can remove theirs

    // Generate field initializers
    let field_inits = fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! {
            #field_name: Default::default()
        }
    });

    let default_impl = quote! {
        impl Default for #struct_name {
            fn default() -> Self {
                Self {
                    #(#field_inits),*
                }
            }
        }
    };

    default_impl
}

/// Generates the Layoutable trait implementation for the struct.
fn generate_layoutable_impl(struct_name: &Ident) -> proc_macro2::TokenStream {
    let trait_impl = quote! {
        impl render_layout::InternalLayoutable for #struct_name {
            fn get_width(&self) -> u32 {
                self.width
            }

            fn set_width(&mut self, width: u32) {
                self.width = width;
            }

            fn get_height(&self) -> u32 {
                self.height
            }

            fn set_height(&mut self, height: u32) {
                self.height = height;
            }

            fn get_x(&self) -> u32 {
                self.x
            }

            fn set_x(&mut self, x: u32) {
                self.x = x;
            }

            fn get_y(&self) -> u32 {
                self.y
            }

            fn set_y(&mut self, y: u32) {
                self.y = y;
            }

            fn get_children_mut(&mut self) -> &mut Vec<Box<dyn render_layout::InternalLayoutable>> {
                &mut self.children
            }

            fn new() -> Self where Self: Sized {
                let mut instance = Self::default();
                instance.children = instance.children();
                instance
            }

            fn as_layoutable(&mut self) -> &mut dyn render_layout::InternalLayoutable {
                self
            }

            fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
                self
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    };

    trait_impl
}
