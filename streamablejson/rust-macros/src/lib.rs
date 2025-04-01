use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use streamablejson::{deserializer::deserialize_orderedbag_from_string, StreamableJSONEntry};


#[derive(Debug)]
struct Wrapper(StreamableJSONEntry);

fn token(entry: &StreamableJSONEntry) -> TokenStream {
    match entry {
        StreamableJSONEntry::Object(obj) => {
            let arr: Vec<TokenStream> = obj.iter().map(|x| {
                let k = token(x.0);
                let v = token(x.1);
                quote! {
                    d.push(#k,#v);
                }
            }).collect();
            quote! {
                {
                    let mut d = OrderedBag::new();
                    #(#arr)*;
                    StreamableJSONEntry::Object(d)
                }
            }
        }
        StreamableJSONEntry::Array(arr) => {
                let arr: Vec<TokenStream> = arr.iter().map(|x| token(x)).collect();
                quote! {
                    StreamableJSONEntry::Array(
                        vec![#(#arr),*]
                    )
                }
            }
        StreamableJSONEntry::String(s) => {
                quote! {
                    StreamableJSONEntry::String(#s.into())
                }
            }
        StreamableJSONEntry::Constant(s) => {
                quote! {
                    StreamableJSONEntry::Constant(#s.into())
                }
            }
        StreamableJSONEntry::Type(_, items) => todo!(),
    }
}


impl ToTokens for Wrapper {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let s = token(&self.0);
        let tok = quote! {
            {
                use streamablejson::StreamableJSONEntry;
                use dataflowgrid_commons::orderedbag::OrderedBag;

                #s
            }
        };
        tokens.extend(tok);
    }
}

#[proc_macro]
pub fn streamablejson(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let text: String = input.to_string();
    let des = Wrapper(deserialize_orderedbag_from_string(text).unwrap());
    quote! {
        #des
    }.into()
}

