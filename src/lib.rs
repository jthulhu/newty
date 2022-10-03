use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token::Comma,
    GenericArgument, PathArguments, PathSegment, Token, Type, WhereClause,
    ItemFn, Signature
};

// #[cfg(test)]
// mod tests {
//     use super::newtype;

//     #[test]
//     fn simple() {
// 	newtype!{
// 	    Hello(Vec<i32>);
// 	}

// 	let a: Hello = vec![3, 2, 1].into();
// 	assert_eq!(a.0, vec![3, 2, 1]);
//     }

//     #[test]
//     fn generics() {
// 	newtype!{
// 	    Hello<'a, T, U>(Vec<&'a (T, U)>);
// 	}

// 	let a: Hello<_, _> = vec![&(3, ()), &(2, ()), &(1, ())].into();
// 	assert_eq!(a.0, vec![&(3, ()), &(2, ()), &(1, ())]);
//     }

//     #[test]
//     fn vec_generic() {
// 	newtype!{
// 	    Foo<T>(Vec<T>)[usize] -> T;

// 	    impl {
// 		fn len(&self) -> usize;
// 		fn capacity(&self) -> usize;
// 		fn push(&mut self, value: T);
// 		fn push_two(&mut self, first: T, second: T) {
// 		    self.push(first);
// 		    self.push(second);
// 		}
// 	    }
// 	}

// 	let mut a: Foo<_> = vec![1, 2, 3].into();
// 	assert_eq!(a.len(), 3);
// 	assert_eq!(a.capacity(), 3);
// 	a.push(4);
// 	assert_eq!(a.len(), 4);
//     }
// }

mod keywords {
    use syn::custom_keyword;

    custom_keyword!(with);
    custom_keyword!(newtype);
}

#[derive(Debug)]
struct NewtypeInfo {
    nt_type: PathSegment,
    nt_type_generics: Option<Punctuated<GenericArgument, Comma>>,
    nt_where: Option<WhereClause>,
    interior_type: Type,
    impls: Vec<ImplInfo>,
}

impl Parse for NewtypeInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
	input.parse::<keywords::newtype>()?;
        let nt_type: PathSegment = input.parse()?;
        let nt_type_generics = if let PathArguments::AngleBracketed(ref args) =
            nt_type.arguments
        {
            Some(args.args.clone())
        } else {
            None
        };
        input.parse::<Token![=]>()?;
        let interior_type = input.parse()?;
        let nt_where = if input.peek(Token![where]) {
            Some(input.parse()?)
        } else {
            None
        };
	let impls = if input.parse::<Token![;]>().is_ok() {
	    Vec::new()
	} else {
	    input.parse::<keywords::with>()?;
	    input.parse::<Impls>()?.impls
	};
	Ok(NewtypeInfo {
	    nt_type,
	    nt_type_generics,
	    nt_where,
	    interior_type,
	    impls,
        })
    }
}

struct Impls {
    impls: Vec<ImplInfo>,
}

impl Parse for Impls {
    fn parse(input: ParseStream) -> syn::Result<Self> {
	let mut impls = Vec::new();
	while let Ok(impl_info) = input.parse::<ImplInfo>() {
	    impls.push(impl_info);
	}
	Ok(Self { impls })
    }
}

#[derive(Debug)]
enum ImplInfo {
    Impl(ItemFn),
    Through(Signature),
    Vec,
    Slice,
    Index,
    Map,
}

impl Parse for ImplInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
	if let Ok(func) = input.parse::<ItemFn>() {
	    Ok(Self::Impl(func))
	} else if let Ok(decl) = input.parse::<Signature>() {
	    input.parse::<Token![;]>()?;
	    Ok(Self::Through(decl))
	} else {
	    todo!()
	}
    }
}

impl ToTokens for ImplInfo {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
	match self {
	    Self::Impl(func) => tokens.extend(quote!{
		impl 
	    }),
	    Self::Through(sign) => {
		sign.to_tokens(tokens)
	    }
	    Self::Vec => todo!(),
	    Self::Slice => todo!(),
	    Self::Index => todo!(),
	    Self::Map => todo!(),
	}
    }
}

#[proc_macro]
pub fn newty(input: TokenStream) -> TokenStream {
    let NewtypeInfo {
        nt_type,
        nt_type_generics,
        nt_where,
        interior_type,
	impls,
    } = parse_macro_input!(input as _);
    let mut tokens = quote! {
	struct #nt_type #nt_where {
	    inner: #interior_type
	}
    };
    for impl_info in impls {
	let args = &nt_type.arguments;
	match impl_info {
	    ImplInfo::Impl(func) => tokens.extend(quote! {
		impl #args #nt_type {
		    #func
		}
	    }),
	    ImplInfo::Through(sign) => {
		let fname = &sign.ident;
		let args = &sign.inputs;
		tokens.extend(quote! {
		    impl #args #nt_type {
			#sign {
			    self.#fname(#args)
			}
		    }
		})
	    }
	    _ => todo!(),
	}
    }
    tokens.into()
}

#[proc_macro_attribute]
pub fn through(args: TokenStream, input: TokenStream) -> TokenStream {
    input
}

// #[macro_export]
// macro_rules! newtype {
//     (@implmthd ($impl_vis:vis)) => {};

//     (@implmthd ($impl_vis:vis)
//      fn $name:ident(&self $(,)? $($arg:ident : $arg_type:ty),*) $(-> $ret_type:tt)?;
//      $($methods:tt)*) => {
// 	$impl_vis fn $name(&self, $($arg: $arg_type),*) $(-> $ret_type)? {
// 	    self.0.$name($($arg),*)
// 	}
// 	newtype!{@implmthd ($impl_vis) $($methods)*}
//     };

//     (@implmthd ($impl_vis:vis)
//      fn $name:ident(self $(,)? $($arg:ident : $arg_type:ty),*) $(-> $ret_type:tt)?;
//      $($methods:tt)*) => {
// 	$impl_vis fn $name(self, $($arg: $arg_type),*) $(-> $ret_type)? {
// 	    self.0.$name($($arg),*)
// 	}
// 	newtype!{@implmthd ($impl_vis) $($methods)*}
//     };

//     (@implmthd ($impl_vis:vis)
//      fn $name:ident(&mut self $(,)? $($arg:ident : $arg_type:ty),*) $(-> $ret_type:tt)?;
//      $($methods:tt)*) => {
// 	$impl_vis fn $name(&mut self, $($arg: $arg_type),*) $(-> $ret_type)? {
// 	    self.0.$name($($arg),*)
// 	}
// 	newtype!{@implmthd ($impl_vis) $($methods)*}
//     };

//     (@implmthd ($impl_vis:vis) $func:item $($methods:tt)*) => {
// 	$impl_vis $func
// 	newtype!{@implmthd ($impl_vis) $($methods)*}
//     };

//     (@impl{$($gen_type:tt),*} $name:ident) => {};

//     (@impl{$($gen_type:tt),*} $name:ident
//      {($impl_vis:vis) { $($methods:tt)* }}
//      $($others:tt)*) => {
// 	impl<$($gen_type),*> $name<$($gen_type),*> {
// 	    newtype!{@implmthd ($impl_vis) $($methods)*}
// 	}
// 	newtype!{@impl{$($gen_type),*} $name $($others)*}
//     };

//     (@implidx {$($gen_type:tt),*} $name:ident) => {};

//     (@implidx {$($gen_type:tt),*}
//      $name:ident [$indexer:ty] $output_type:ty) => {
// 	impl<$($gen_type),*> ::std::ops::Index<$indexer> for $name<$($gen_type),*> {
// 	    type Output = $output_type;

// 	    fn index(&self, index: $indexer) -> &Self::Output {
// 		self.0.index(index)
// 	    }
// 	}
//     };

//     ($(#[$($meta:meta),*])* $visibility:vis $name:ident
//      $(<$($gen_ty pe:tt),*>)?
//      ($interior_type:ty)
//      $([$index_type:ty] -> $output_type:ty)?;
//      $($impl_vis:vis impl {
// 	 $($methods:tt)*
//      })*) => {
// 	$(#[$($meta),*])*
// 	#[repr(transparent)]
// 	$visibility struct $name $(<$($gen_type),*>)?($interior_type);

// 	newtype!{
// 	    @implidx {$($($gen_type),*)?}
// 		$name $([$index_type] $output_type)?
// 	}

// 	newtype!{
// 	    @impl{$($($gen_type),*)?} $name $({
// 		($impl_vis) { $($methods)* }
// 	    })*
// 	}

// 	impl $(<$($gen_type),*>)? From<$interior_type> for $name $(<$($gen_type),*>)? {
// 	    $visibility fn from(int: $interior_type) -> Self {
// 		Self(int)
// 	    }
// 	}
//     };
// }
