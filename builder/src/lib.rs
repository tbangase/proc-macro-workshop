use proc_macro::TokenStream;
use syn::{
    self, DeriveInput, Data, 
    Fields, FieldsNamed, Type, 
    PathArguments, GenericArgument
};
use quote::{quote, format_ident};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;
    let builder_name = format_ident!("{struct_name}Builder");
    let fields = extract_fields(&input.data);

    let field_iter = fields.named.iter().map(|field| {
        let ident = field.ident.clone();
        let ty = field.ty.clone();
        if is_option_type(&field.ty) {
            quote! {
                #ident: #ty
            }
        } else {
            quote! {
                #ident: Option<#ty>
            }
        }
    });

    let initial_field_iter = fields.named.iter().map(|field| {
        let ident = field.ident.clone();
        quote! {
            #ident: None
        }
    });

    let build_field_iter = fields.named.iter().map(|field| {
        let ident = field.ident.clone();
        if is_option_type(&field.ty) {
            quote! {
                #ident: self.#ident.clone()
            }
        } else {
            quote! {
                #ident: self.#ident.clone().ok_or(format!("Field {} not allowed None", stringify!(#ident)))?
            }
        }
    });

    let setter_iter = fields.named.iter().map(|field| {
        let ident = field.ident.clone();
        let ty = field.ty.clone();
        // let wrapped_type = false;
        if is_option_type(&field.ty) {
            if let Some(wrapped_type) = unwrap_type(&ty) {
                quote! {
                    pub fn #ident(&mut self, #ident: #wrapped_type) -> &mut Self {
                        self.#ident = Some(#ident);
                        self
                    }
                }
            } else {
                panic!("Implementation cannot handle Option type. Code again")
            }
        } else {
            quote! {
                pub fn #ident(&mut self, #ident: #ty) -> &mut Self {
                    self.#ident = Some(#ident);
                    self
                }
            }
        }
    });

    let expanded = quote! {
        pub struct #builder_name {
            #(#field_iter),*
        }

        impl #struct_name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#initial_field_iter),*
                }
            }
        }

        impl #builder_name {
            #(#setter_iter)*

            pub fn build(&self) -> Result<#struct_name, Box<dyn std::error::Error>> {
                Ok(#struct_name {
                    #(#build_field_iter),*
                })
            }
        }
    };

    TokenStream::from(expanded)
}

fn extract_fields(data: &Data) -> &FieldsNamed {
    match &*data {
        Data::Struct(struct_data) => match &struct_data.fields {
            Fields::Named(fields) => &fields,
            _ => panic!("This macro is not supported for UnNamed or Unit Structures")
        },
        _ => panic!("This macro is only supported for Structures")
    }
}

fn is_option_type(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => {
            if let Some(path_segment) = type_path.path.segments.iter().next() {
                if path_segment.ident == "Option" {
                    return true
                }
            };
        },
        _ => panic!("unexpected field type")
    }
    false
}

fn unwrap_type(ty: &Type) -> Option<&Type> {
    match ty {
        Type::Path(type_path) => {
            if let Some(path_segment) = type_path.path.segments.iter().next() {
                match &path_segment.arguments {
                    PathArguments::AngleBracketed(gen_args) => {
                        match gen_args.args.last()? {
                            GenericArgument::Type(ty) => {
                                return Some(&ty)
                            },
                            _ => return None 
                        }
                    },
                    _ => return None 
                }
            };
        },
        _ => return None 
    }
    None
}

