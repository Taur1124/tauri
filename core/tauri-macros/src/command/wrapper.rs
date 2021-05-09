// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
  parse::{Parse, ParseBuffer},
  parse_macro_input,
  spanned::Spanned,
  FnArg, Ident, ItemFn, Pat, Token, Visibility,
};

/// The execution context of the command.
enum ExecutionContext {
  Async,
  Blocking,
}

impl Parse for ExecutionContext {
  fn parse(input: &ParseBuffer) -> syn::Result<Self> {
    if input.is_empty() {
      return Ok(Self::Blocking);
    }

    input
      .parse::<Token![async]>()
      .map(|_| Self::Async)
      .map_err(|_| {
        syn::Error::new(
          input.span(),
          "only a single item `async` is currently allowed",
        )
      })
  }
}

/// Create a new [`Wrapper`] from the function and the generated code parsed from the function.
pub fn wrapper(attributes: TokenStream, item: TokenStream) -> TokenStream {
  let function = parse_macro_input!(item as ItemFn);
  let wrapper = super::format_command_wrapper(&function.sig.ident);
  let visibility = &function.vis;

  // macros used with `pub use my_macro;` need to be exported with `#[macro_export]`
  let maybe_macro_export = match &function.vis {
    Visibility::Public(_) => quote!(#[macro_export]),
    _ => Default::default(),
  };

  // body to the command wrapper or a `compile_error!` of an error occurred while parsing it.
  let body = syn::parse::<ExecutionContext>(attributes)
    .map(|context| match function.sig.asyncness {
      Some(_) => ExecutionContext::Async,
      None => context,
    })
    .and_then(|context| match context {
      ExecutionContext::Async => body_async(&function),
      ExecutionContext::Blocking => body_blocking(&function),
    })
    .unwrap_or_else(syn::Error::into_compile_error);

  // Rely on rust 2018 edition to allow importing a macro from a path.
  quote!(
    #function

    #maybe_macro_export
    macro_rules! #wrapper {
        // double braces because the item is expected to be a block expression
        ($path:path, $invoke:ident) => {{
          // import all the autoref specialization items
          #[allow(unused_imports)]
          use ::tauri::command::private::*;

          // prevent warnings when the body is a `compile_error!` or if the command has no arguments
          #[allow(unused_variables)]
          let ::tauri::Invoke { message, resolver } = $invoke;

          #body
      }};
    }

    // allow the macro to be resolved with the same path as the command function
    #[allow(unused_imports)]
    #visibility use #wrapper;
  )
  .into()
}

/// Generates an asynchronous command response from the arguments and return value of a function.
///
/// See the [`tauri::command`] module for all the items and traits that make this possible.
///
/// * Requires binding `message` and `resolver`.
/// * Requires all the traits from `tauri::command::private` to be in scope.
///
/// [`tauri::command`]: https://docs.rs/tauri/*/tauri/runtime/index.html
fn body_async(function: &ItemFn) -> syn::Result<TokenStream2> {
  parse_args(function).map(|args| {
    quote! {
      resolver.respond_async_serialized(async move {
        let result = $path(#(#args?),*);
        (&result).async_kind().future(result).await
      })
    }
  })
}

/// Generates a blocking command response from the arguments and return value of a function.
///
/// See the [`tauri::command`] module for all the items and traits that make this possible.
///
/// * Requires binding `message` and `resolver`.
/// * Requires all the traits from `tauri::command::private` to be in scope.
///
/// [`tauri::command`]: https://docs.rs/tauri/*/tauri/runtime/index.html
fn body_blocking(function: &ItemFn) -> syn::Result<TokenStream2> {
  let args = parse_args(function)?;

  // the body of a `match` to early return any argument that wasn't successful in parsing.
  let match_body = quote!({
    Ok(arg) => arg,
    Err(err) => return resolver.invoke_error(err),
  });

  Ok(quote! {
    let result = $path(#(match #args #match_body),*);
    (&result).blocking_kind().block(result, resolver);
  })
}

/// Parse all arguments for the command wrapper to use from the signature of the command function.
fn parse_args(function: &ItemFn) -> syn::Result<Vec<TokenStream2>> {
  function
    .sig
    .inputs
    .iter()
    .map(|arg| parse_arg(&function.sig.ident, arg))
    .collect()
}

/// Transform a [`FnArg`] into a command argument.
///
/// * Requires binding `message`.
fn parse_arg(command: &Ident, arg: &FnArg) -> syn::Result<TokenStream2> {
  // we have no use for self arguments
  let mut arg = match arg {
    FnArg::Typed(arg) => arg.pat.as_ref().clone(),
    FnArg::Receiver(arg) => {
      return Err(syn::Error::new(
        arg.span(),
        "unable to use self as a command function parameter",
      ))
    }
  };

  // we only support patterns that allow us to extract some sort of keyed identifier.
  let key = match &mut arg {
    Pat::Ident(arg) => arg.ident.to_string(),
    Pat::Wild(_) => "_".into(),
    Pat::Struct(s) => super::path_to_command(&mut s.path).ident.to_string(),
    Pat::TupleStruct(s) => super::path_to_command(&mut s.path).ident.to_string(),
    err => {
      return Err(syn::Error::new(
        err.span(),
        "only named, wildcard, struct, and tuple struct arguments allowed",
      ))
    }
  };

  // also catch self arguments that use FnArg::Typed syntax
  if key == "self" {
    return Err(syn::Error::new(
      key.span(),
      "unable to use self as a command function parameter",
    ));
  }

  let key = snake_case_to_camel_case(key);

  Ok(quote!(::tauri::command::CommandArg::from_command(
    ::tauri::command::CommandItem {
      name: stringify!(#command),
      key: #key,
      message: &message,
    }
  )))
}

fn snake_case_to_camel_case(s: String) -> String {
  if s.as_str().contains('_') {
    let mut camel = String::with_capacity(s.len());
    let mut to_upper = false;
    for c in s.chars() {
      match c {
        '_' => to_upper = true,
        c if to_upper => {
          camel.push(c.to_ascii_uppercase());
          to_upper = false;
        }
        c => camel.push(c),
      }
    }

    camel
  } else {
    s
  }
}
