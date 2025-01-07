use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{ToTokens, format_ident, quote};
/**
  WgslVecInput::vec_len::<Position>() becomes `POSITION_INPUT_ARRAY_LENGTH``
 WgslVecInput::vec_val::<Radius>( SOMETHING ) becomes `RADIUS_INPUT_ARRAY[ SOMETHING ]``
 WgslOutput::push::<CollisionResult>( SOMETHING ) becomes:
 ```rust
 let collisionresult_index = atomicAdd(&collisionresult_counter,1u);
 if collisionresult_index < COLLISIONRESULT_OUTPUT_ARRAY_LENGTH {
    collisionresult_output_array[ collisionresult_index ] = SOMETHING;
    }
    ```
and WgslOutput::len::<CollisionResult>() becomes `COLLISIONRESULT_OUTPUT_ARRAY_LENGTH`
and WgslOutput::set::<CollisionResult>(SOMETHING1, SOMETHING2) becomes `collisionresult_output_array[ SOMETHING1 ] = SOMETHING2`
 */
use syn::{
    AngleBracketedGenericArguments, Expr, ExprCall, ExprMethodCall, GenericArgument, Ident,
    ItemMod, Path, PathArguments, Type, parse_quote, parse_str,
    visit_mut::{self, VisitMut},
};

// module -> item -> block -> stmt -> expr -> methodcall -> args -> type -> path -> segment -> ident

enum WgslHelperCategory {
    VecInput,
    Output,
    _Invalid,
}
// from ident
impl WgslHelperCategory {
    fn from_ident(ident: Ident) -> Option<Self> {
        match ident.to_string().as_str() {
            "WgslVecInput" => Some(WgslHelperCategory::VecInput),
            "WgslOutput" => Some(WgslHelperCategory::Output),
            _ => None,
        }
    }
}
enum WgslHelperMethod {
    VecLen,
    VecVal,
    Push,
    Len,
    Set,
    _Invalid,
}
impl WgslHelperMethod {
    fn from_ident(ident: Ident) -> Option<Self> {
        match ident.to_string().as_str() {
            "vec_len" => Some(WgslHelperMethod::VecLen),
            "vec_val" => Some(WgslHelperMethod::VecVal),
            "push" => Some(WgslHelperMethod::Push),
            "len" => Some(WgslHelperMethod::Len),
            "set" => Some(WgslHelperMethod::Set),
            _ => None,
        }
    }
}

struct WgslUserDefinedType {
    pub name: Ident,
    pub upper: Ident,
    pub lower: Ident,
}
impl WgslUserDefinedType {
    pub fn new(name: &Ident) -> Self {
        let upper = Ident::new(&name.to_string().to_uppercase(), name.span());
        let lower = Ident::new(&name.to_string().to_lowercase(), name.span());
        Self {
            name: name.clone(),
            upper,
            lower,
        }
    }
    pub fn input_array_length(&self) -> Ident {
        format_ident!("{}_INPUT_ARRAY_LENGTH", self.upper)
    }
    pub fn input_array(&self) -> Ident {
        format_ident!("{}_input_array", self.lower)
    }
    pub fn output_array_length(&self) -> Ident {
        format_ident!("{}_OUTPUT_ARRAY_LENGTH", self.upper)
    }
    pub fn output_array(&self) -> Ident {
        format_ident!("{}_output_array", self.lower)
    }
    pub fn counter(&self) -> Ident {
        format_ident!("{}_counter", self.lower)
    }
    pub fn index(&self) -> Ident {
        format_ident!("{}_output_array_index", self.lower)
    }
}
struct WgslHelperMethodReplacer {}
impl WgslHelperMethodReplacer {
    fn input_len(type_name: WgslUserDefinedType) -> TokenStream {
        type_name.input_array_length().to_token_stream()
    }
    fn input_val(type_name: WgslUserDefinedType, index: TokenStream) -> TokenStream {
        let name = type_name.input_array();
        quote! {
            #name [ #index ]
        }
    }
    fn output_push(type_name: WgslUserDefinedType, value: TokenStream) -> TokenStream {
        let counter = type_name.counter();
        let arr = type_name.output_array();
        let len = type_name.output_array_length();
        let index = type_name.index();
        quote! {
            {
            let #index = atomicAdd( & #counter, 1u);
            if #index < #len {
              #arr [ #index ] = #value;
            }
            }
        }
    }
    fn output_len(type_name: WgslUserDefinedType) -> TokenStream {
        let len = type_name.output_array_length();
        len.to_token_stream()
    }
    fn output_set(
        type_name: WgslUserDefinedType,
        index: TokenStream,
        value: TokenStream,
    ) -> TokenStream {
        let arr = type_name.output_array();
        quote! {
            #arr [ #index ] = #value
        }
    }
}

struct WgslHelperMethodMatcher {}
impl WgslHelperMethodMatcher {
    fn convert(
        category: WgslHelperCategory,
        method: WgslHelperMethod,
        type_name: WgslUserDefinedType,
        arg1: Option<&Expr>,
        arg2: Option<&Expr>,
    ) -> Option<TokenStream> {
        match (category, method) {
            (WgslHelperCategory::VecInput, WgslHelperMethod::VecLen) => {
                Some(WgslHelperMethodReplacer::input_len(type_name))
            }
            (WgslHelperCategory::VecInput, WgslHelperMethod::VecVal) => {
                assert!(
                    arg1.is_some(),
                    "Expected an argument for input vec value getter"
                );
                Some(WgslHelperMethodReplacer::input_val(
                    type_name,
                    arg1.to_token_stream(),
                ))
            }
            (WgslHelperCategory::Output, WgslHelperMethod::Push) => {
                assert!(arg1.is_some(), "Expected an argument for output push");
                Some(WgslHelperMethodReplacer::output_push(
                    type_name,
                    arg1.to_token_stream(),
                ))
            }
            (WgslHelperCategory::Output, WgslHelperMethod::Len) => {
                Some(WgslHelperMethodReplacer::output_len(type_name))
            }
            (WgslHelperCategory::Output, WgslHelperMethod::Set) => {
                assert!(
                    arg1.is_some() && arg2.is_some(),
                    "Expected two arguments for output set"
                );
                Some(WgslHelperMethodReplacer::output_set(
                    type_name,
                    arg1.to_token_stream(),
                    arg2.to_token_stream(),
                ))
            }
            _ => None,
        }
    }
}

fn get_special_function_category(call: &ExprCall) -> Option<WgslHelperCategory> {
    if let Expr::Path(path) = &*call.func {
        if let Some(first_seg) = path.path.segments.first() {
            return WgslHelperCategory::from_ident(first_seg.ident.clone());
        }
    }
    None
}
fn get_special_function_method(call: &ExprCall) -> Option<WgslHelperMethod> {
    if let Expr::Path(path) = &*call.func {
        if let Some(last_seg) = path.path.segments.last() {
            return WgslHelperMethod::from_ident(last_seg.ident.clone());
        }
    }
    None
}
fn get_special_function_generic_type(call: &ExprCall) -> Option<WgslUserDefinedType> {
    if let Expr::Path(path) = &*call.func {
        if let Some(last_seg) = path.path.segments.last() {
            if let PathArguments::AngleBracketed(args) = &last_seg.arguments {
                if let Some(GenericArgument::Type(Type::Path(type_path))) = args.args.first() {
                    if let Some(last_seg) = type_path.path.segments.last() {
                        return Some(WgslUserDefinedType::new(&last_seg.ident));
                    }
                }
            }
        }
    }
    None
}

fn replace(call: ExprCall) -> Option<TokenStream> {
    let category = get_special_function_category(&call);
    let method = get_special_function_method(&call);
    let type_name = get_special_function_generic_type(&call);
    if let Some(cat) = category {
        if let Some(met) = method {
            if let Some(ty) = type_name {
                return WgslHelperMethodMatcher::convert(
                    cat,
                    met,
                    ty,
                    call.args.first(),
                    call.args.get(1),
                );
            }
        }
    }
    None
}

struct HelperFunctionConverter;

impl VisitMut for HelperFunctionConverter {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        visit_mut::visit_expr_mut(self, expr);
        if let Expr::Call(call) = expr {
            let replacement = replace(call.clone());
            if let Some(r) = replacement {
                *expr = parse_quote!(#r);
            }
        }
    }
}

pub fn convert_special_helper_functions(module: &ItemMod) -> ItemMod {
    let mut module = module.clone();
    HelperFunctionConverter.visit_item_mod_mut(&mut module);
    module
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;
    use syn::parse_quote;

    #[test]
    fn test_vec_len() {
        let input: ItemMod = parse_quote! {
            mod test {
                fn example() {
                    let x = WgslVecInput::vec_len::<Position>();
                }
            }
        };
        let expected_output =
            "mod test { fn example () { let x = POSITION_INPUT_ARRAY_LENGTH ; } }";

        let output = convert_special_helper_functions(&input);
        let result = output.to_token_stream().to_string();
        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }

    #[test]
    fn test_vec_val() {
        let input: ItemMod = parse_quote! {
            mod test {
                fn example() {
                    let x = WgslVecInput::vec_val::<Radius>(5);
                }
            }
        };
        let expected_output = "mod test { fn example () { let x = radius_input_array [5] ; } }";

        let output = convert_special_helper_functions(&input);
        let result = output.to_token_stream().to_string();
        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }

    #[test]
    fn test_push() {
        let input: ItemMod = parse_quote! {
            mod test {
                fn example() {
                    WgslOutput::push::<CollisionResult>(value);
                }
            }
        };

        let expected_output = "mod test { fn example () { { let collisionresult_output_array_index = atomicAdd (& collisionresult_counter , 1u) ; if collisionresult_output_array_index < COLLISIONRESULT_OUTPUT_ARRAY_LENGTH { collisionresult_output_array [collisionresult_output_array_index] = value ; } } ; } }";
        let output = convert_special_helper_functions(&input);
        let result = output.to_token_stream().to_string();
        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }

    #[test]
    fn test_output_len() {
        let input: ItemMod = parse_quote! {
            mod test {
                fn example() {
                    let x = WgslOutput::len::<CollisionResult>();
                }
            }
        };

        let expected_output =
            "mod test { fn example () { let x = COLLISIONRESULT_OUTPUT_ARRAY_LENGTH ; } }";

        let output = convert_special_helper_functions(&input);
        let result = output.to_token_stream().to_string();
        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }

    #[test]
    fn test_output_set() {
        let input: ItemMod = parse_quote! {
            mod test {
                fn example() {
                    WgslOutput::set::<CollisionResult>(idx, val);
                }
            }
        };
        let expected_output =
            "mod test { fn example () { collisionresult_output_array [idx] = val ; } }";

        let output = convert_special_helper_functions(&input);
        let result = output.to_token_stream().to_string();
        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }
}
