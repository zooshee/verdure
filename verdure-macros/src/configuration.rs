use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Expr, Fields, Lit};

pub(crate) fn impl_configuration_derive(input: &DeriveInput) -> TokenStream {
    let struct_name = &input.ident;
    let config_module_key = parse_config_module_key(&input.attrs)
        .expect("Missing required #[configuration(\"...\")] attribute.");
    let field_setters = generate_field_setters(&input.data, &config_module_key);
    let struct_init = generate_struct_initialization(&input.data);

    let expanded = quote! {
        impl ::verdure::config::ConfigInitializer for #struct_name {
            fn from_config_manager(config_manager: std::sync::Arc<::verdure::config::ConfigManager>) -> ::verdure::ContextResult<Self> {
                let mut instance = Self {
                    #(#struct_init)*
                };
                #(#field_setters)*
                Ok(instance)
            }

            fn config_module_key() -> &'static str {
                #config_module_key
            }
        }
        ::inventory::submit! {
            ::verdure::config::ConfigFactory {
                type_id: || std::any::TypeId::of::<#struct_name>(),
                create_fn: |config_manager| {
                    let instance = <#struct_name as ::verdure::config::ConfigInitializer>::from_config_manager(config_manager)?;
                    Ok(std::sync::Arc::new(instance))
                },
            }
        }
    };

    TokenStream::from(expanded)
}

fn generate_struct_initialization(data: &Data) -> Vec<TokenStream> {
    let mut initializers = Vec::new();

    if let Data::Struct(data_struct) = data {
        if let Fields::Named(fields) = &data_struct.fields {
            for field in &fields.named {
                let field_ident = field.ident.as_ref().unwrap();

                let mut has_default = false;
                let mut default_value: Option<TokenStream> = None;

                for attr in &field.attrs {
                    if attr.path().is_ident("config_default") {
                        if let Ok(lit) = attr.parse_args::<Lit>() {
                            default_value = Some(match lit {
                                Lit::Int(int_lit) => quote! { Some(#int_lit) },
                                Lit::Str(str_lit) => quote! { Some(#str_lit.to_string()) },
                                Lit::Bool(bool_lit) => quote! { Some(#bool_lit) },
                                Lit::Float(float_lit) => quote! { Some(#float_lit) },
                                other => quote! { Some(#other) },
                            });
                            has_default = true;
                        }
                    } else if attr.path().is_ident("config_default_t") {
                        if let Ok(expr) = attr.parse_args::<Expr>() {
                            default_value = Some(quote! { #expr });
                            has_default = true;
                        }
                    }
                }


                let field_init = if has_default {
                    let default_val = default_value.unwrap();
                    quote! { #field_ident: #default_val, }
                } else {
                    // No default attribute - use None for Option fields
                    quote! { #field_ident: None, }
                };

                initializers.push(field_init);
            }
        }
    }

    initializers
}

fn generate_field_setters(data: &Data, config_module_key: &String) -> Vec<TokenStream> {
    let mut setters = Vec::new();
    if let Data::Struct(data_struct) = data {
        if let Fields::Named(fields) = &data_struct.fields {
            for field in &fields.named {
                let field_ident = field.ident.as_ref().unwrap();
                let field_name = field_ident.to_string();
                let config_key = format!("{}.{}", config_module_key, field_name);

                let setter = quote! {
                    if let Some(config_value) = config_manager.get(#config_key) {
                        // TODO: as_string change as_any_type
                        if let Some(str_val) = config_value.as_string() {
                            if let Ok(parsed_val) = str_val.parse() {
                                instance.#field_ident = Some(parsed_val);
                            }
                        }
                    }
                };
                setters.push(setter);
            }
        }
    }
    setters
}

fn parse_config_module_key(attrs: &Vec<Attribute>) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("configuration") {
            if let Ok(meta_list) = attr.meta.require_list() {
                if let Ok(Lit::Str(lit_str)) = syn::parse2::<Lit>(meta_list.tokens.clone()) {
                    return Some(lit_str.value());
                }
            }
        }
    }
    None
}
