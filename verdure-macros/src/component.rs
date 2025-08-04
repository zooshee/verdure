use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Error, Field, Fields, FieldsNamed, GenericArgument,
    PathArguments, Type,
};

pub(crate) fn impl_component_derive(ast: &DeriveInput) -> TokenStream {
    let struct_name = &ast.ident;

    let fields = match validate_struct_input(&ast) {
        Ok(fields) => fields,
        Err(err) => return err.to_compile_error(),
    };

    let (attr_fields, non_attr_fields) = partition_fields(fields);

    match process_fields(&attr_fields, &non_attr_fields, struct_name, &ast.attrs) {
        Ok(expanded) => TokenStream::from(expanded),
        Err(err) => err.to_compile_error(),
    }
}

fn process_fields(
    attr_fields: &Vec<Field>,
    non_attr_fields: &Vec<Field>,
    struct_name: &Ident,
    attrs: &Vec<Attribute>,
) -> Result<TokenStream, Error> {
    let dependency_inner_types = match extract_dependency_types(attr_fields) {
        Ok(types) => types,
        Err(err) => return Err(err),
    };

    let autowired_names: Vec<_> = attr_fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect();

    let non_autowired_initializers: Vec<_> = non_attr_fields
        .iter()
        .map(|f| {
            let name = f.ident.as_ref().unwrap();
            if is_optional_field(&f.ty) {
                quote! { #name: None }
            } else {
                quote! { #name: Default::default() }
            }
        })
        .collect();

    let scope =
        find_scope_attribute(attrs).unwrap_or(quote! { ::verdure::ComponentScope::Singleton });

    let expanded = quote! {
        impl ::verdure::ComponentInitializer for #struct_name {
            type Dependencies = ( #( std::sync::Arc<#dependency_inner_types>, )* );
            fn __new(deps: Self::Dependencies) -> Self {
                let ( #( #autowired_names, )* ) = deps;

                Self {
                     #( #autowired_names, )*
                    #( #non_autowired_initializers, )*
                }
            }

            fn __scope() -> ::verdure::ComponentScope {
                #scope
            }
        }


        inventory::submit! {
            ::verdure::ComponentDefinition {
                type_id: || std::any::TypeId::of::<#struct_name>(),
                type_name: stringify!(#struct_name),
                scope: || <#struct_name as ::verdure::ComponentInitializer>::__scope(),
                dependencies: || vec![
                        #( std::any::TypeId::of::<#dependency_inner_types>(), )*
                    ],
                creator: |deps: std::collections::HashMap<std::any::TypeId, ::verdure::ComponentInstance>| -> Result<::verdure::ComponentInstance, ::verdure::error::component::ComponentError> {
                    // Ok(#struct_name as ::verdure::Component::__new())
                    #(
                        let #autowired_names:std::sync::Arc<#dependency_inner_types> = deps.get(&std::any::TypeId::of::<#dependency_inner_types>())
                        .ok_or_else(|| ::verdure::error::component::ComponentError::DependencyNotFound(
                            format!("Dependency '{}' not found in provided deps", stringify!(#dependency_inner_types))
                        ))?
                        .clone()
                        .downcast::<#dependency_inner_types>()
                        .map_err(|_| ::verdure::error::component::ComponentError::DowncastFailed(
                            format!("Failed to downcast dependency '{}'", stringify!(#dependency_inner_types))
                        ))?;
                    )*
                    let deps_tuple = (
                        #(
                            #autowired_names,
                        )*
                    );
                    let instance = <#struct_name as ::verdure::ComponentInitializer>::__new(deps_tuple);
                    Ok(std::sync::Arc::new(instance))
                },
            }
        }
    };

    Ok(expanded)
}

fn find_scope_attribute(attrs: &Vec<Attribute>) -> Option<TokenStream> {
    for attr in attrs {
        if attr.path().is_ident("scope") {
            if let Ok(meta) = attr.meta.require_list() {
                let tokens = meta.tokens.clone();
                return Some(quote! { #tokens });
            } else {
                return Some(quote! {
                    verdure::ComponentScope::Singleton
                });
            }
        }
    }
    None
}

fn extract_dependency_types(autowired_fields: &Vec<Field>) -> Result<Vec<Type>, Error> {
    let mut dependency_types = Vec::new();

    for field in autowired_fields {
        let inner_type = extract_arc_inner_type(&field.ty)?;
        dependency_types.push(inner_type);
        //dependency_types.push(field.ty.clone());
    }

    Ok(dependency_types)
}

fn extract_arc_inner_type(ty: &Type) -> Result<Type, Error> {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            // 检查是否为 Arc 类型
            if segment.ident != "Arc" {
                return Err(Error::new_spanned(
                    ty,
                    "Fields with #[autowired] attribute must be of type Arc<T>",
                ));
            }

            if let PathArguments::AngleBracketed(args) = &segment.arguments {
                if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                    return Ok(inner_ty.clone());
                }
            }
        }
    }

    Err(Error::new_spanned(
        ty,
        "Fields with #[autowired] attribute must be of type Arc<T>",
    ))
}

fn is_optional_field(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                return true;
            }
        }
    }
    false
}

fn partition_fields(fields: &FieldsNamed) -> (Vec<Field>, Vec<Field>) {
    let mut attr_fields = Vec::new();
    let mut non_attr_fields = Vec::new();

    for field in &fields.named {
        if has_marco_attributes(&field.attrs) {
            attr_fields.push(field.clone());
        } else {
            non_attr_fields.push(field.clone());
        }
    }
    (attr_fields, non_attr_fields)
}

fn has_marco_attributes(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("autowired"))
}

fn validate_struct_input(ast: &DeriveInput) -> Result<&FieldsNamed, Error> {
    match &ast.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(fields) => Ok(fields),
            _ => Err(Error::new_spanned(
                ast,
                "Component derive macro only supports structs with named fields",
            )),
        },
        _ => Err(Error::new_spanned(
            ast,
            "Component derive macro can only be used on structs",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use syn::parse_quote;

    #[test]
    fn test_simple_struct() {
        let input: DeriveInput = parse_quote! {
            #[derive(Component)]
            #[component(name = "xxx", scope="Singleton")]
            struct SimpleStruct {
                #[autowired]
                field1: Arc<DemoStruct>,
                field2: u32,
                field3: i32,
            }
        };

        println!("gen: {}", impl_component_derive(&input))
    }
}
