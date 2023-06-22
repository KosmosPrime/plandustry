use proc_macro::TokenStream;
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn kebab2title(input: TokenStream) -> TokenStream {
    let input_str = parse_macro_input!(input as LitStr).value();

    let converted = kebab2title_impl(&input_str);
    format!("\"{converted}\"").parse().unwrap()
}

fn kebab2title_impl(data: &str) -> String {
    let mut result = String::with_capacity(data.len());
    let mut capitalize_next = true;

    for c in data.chars() {
        if c == '-' {
            result.push(' ');
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}
