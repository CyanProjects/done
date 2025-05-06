// Copyright 2025 Cyan Changes. MIT license.

use std::str::FromStr;
use deno_core::{op2, JsRuntime, ModuleSpecifier, RequestedModuleType};
use deno_core::v8;

deno_core::extension!(done_loader,
  deps = [ ],
  ops = [
    op_loader_cache_keys,
    op_loader_cache_delete,
  ],
  esm = [ "01_loader.ts" ],
);

#[derive(Debug, thiserror::Error, deno_error::JsError)]
pub enum LoaderError {
  #[class(type)]
  #[error("Invalid URL")]
  InvalidUrl
}

#[op2]
pub fn op_loader_cache_keys<'s>(
  scope: &mut v8::HandleScope<'s>,
) -> Result<v8::Local<'s, v8::Value>, LoaderError> {
  let map = unsafe { JsRuntime::module_map_from(scope) };
  let names = map
      .list_modules(&RequestedModuleType::None)
      .into_iter()
      .filter(|name| !(name.starts_with("ext") || name.starts_with("node")))
      .map(|name| name.v8_string(scope).unwrap().cast::<v8::Value>())
      .collect::<Vec<_>>();
  Ok(
    v8::Array::new_with_elements(
      scope,
      names.as_slice()
    ).into()
  )
}

#[op2(fast)]
pub fn op_loader_cache_delete(
  scope: &mut v8::HandleScope,
  #[string] name: &str
) -> Result<bool, LoaderError> {
  let map = unsafe { JsRuntime::module_map_from(scope) };
  Ok(map.delete_module(
    RequestedModuleType::None,
    ModuleSpecifier::from_str(name).map_err(|_| LoaderError::InvalidUrl)?
  ))
}
