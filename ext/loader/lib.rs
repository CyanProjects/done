// Copyright 2025 Cyan Changes. MIT license.

use deno_core::{op2, JsRuntime, ModuleSpecifier, RequestedModuleType, ResolutionKind, SymbolicModule};
use deno_core::error::{CoreError, ModuleLoaderError};
use deno_core::v8;

deno_core::extension!(done_loader,
  deps = [ ],
  ops = [
    op_loader_cache_keys,
    op_loader_cache_entries,
    op_loader_cache_get,
    op_loader_cache_set,
    op_loader_cache_delete,
    op_loader_module_get,
    op_loader_module_requests,
    op_loader_resolve,
  ],
  esm = [ "01_loader.ts" ],
);

#[op2]
pub fn op_loader_cache_entries<'s>(
  scope: &mut v8::HandleScope<'s>,
) -> Result<v8::Local<'s, v8::Value>, ModuleLoaderError> {
  let map = unsafe { JsRuntime::module_map_from(scope) };
  let names = map
      .map_list_modules(
        &RequestedModuleType::None,
        |specifier, id| {
          if specifier.starts_with("ext:") || specifier.starts_with("node") {
            return None
          }
          let ret: [v8::Local<v8::Value>; 2] = [
            specifier.v8_string(scope).unwrap().cast(),
            v8::Number::new(scope, match id {
              SymbolicModule::Alias(mod_name) => map.map_get_module(mod_name.as_str(), &RequestedModuleType::None)?,
              SymbolicModule::Mod(mod_id) => *mod_id
            } as f64).cast()
          ];
          Some(v8::Array::new_with_elements(scope, &ret).into())
        }
      );
  Ok(
    v8::Array::new_with_elements(
      scope,
      names.as_slice()
    ).into()
  )
}

#[op2]
pub fn op_loader_cache_keys<'s>(
  scope: &mut v8::HandleScope<'s>,
) -> Result<v8::Local<'s, v8::Value>, ModuleLoaderError> {
  let map = unsafe { JsRuntime::module_map_from(scope) };
  let specifiers = map
      .map_list_modules(
        &RequestedModuleType::None,
        |name, _id| {
          if name.starts_with("ext:") || name.starts_with("node") {
            return None
          }
          Some(name.v8_string(scope).unwrap().cast())
        }
      );
  Ok(
    v8::Array::new_with_elements(
      scope,
      specifiers.as_slice()
    ).into()
  )
}

#[op2(fast)]
#[smi]
pub fn op_loader_cache_get(
  scope: &mut v8::HandleScope,
  #[string] specifier: &str
) -> Result<usize, ModuleLoaderError> {
  let map = unsafe { JsRuntime::module_map_from(scope) };
  map
      .map_get_module(specifier, &RequestedModuleType::None)
      .ok_or(ModuleLoaderError::NotFound)
}

#[op2(fast)]
#[smi]
pub fn op_loader_cache_set(
  scope: &mut v8::HandleScope,
  #[string] specifier: &str,
  #[smi] module_id: usize
) -> Result<usize, ModuleLoaderError> {
  let map = unsafe { JsRuntime::module_map_from(scope) };
  map
      .map_set_module(specifier, RequestedModuleType::None, module_id)
      .ok_or(ModuleLoaderError::NotFound)
      .map(|symbolic_module| match symbolic_module {
        SymbolicModule::Alias(module_name) => map.map_get_module(&module_name, &RequestedModuleType::None),
        SymbolicModule::Mod(module_id) => Some(module_id)
      })
      .map(|mod_id| mod_id.unwrap_or(0))
}

#[op2(fast)]
pub fn op_loader_cache_delete(
  scope: &mut v8::HandleScope,
  #[string] specifier: &str
) -> Result<bool, ModuleLoaderError> {
  let map = unsafe { JsRuntime::module_map_from(scope) };
  Ok(map.map_delete_module(
    RequestedModuleType::None,
    ModuleSpecifier::parse(specifier).map_err(CoreError::from)?
  ))
}

#[op2]
pub fn op_loader_module_get<'s>(
  scope: &mut v8::HandleScope<'s>,
  #[smi] module_id: usize
) -> Result<v8::Local<'s, v8::Value>, ModuleLoaderError> {
  let map = unsafe { JsRuntime::module_map_from(scope) };
  let module = match map.get_module(scope, module_id) {
    Some(module) => module,
    None => return Err(ModuleLoaderError::NotFound)
  };

  if module.get_status() != v8::ModuleStatus::Evaluated {
    return Err(ModuleLoaderError::NotFound)
  }

  Ok(v8::Local::new(scope, module.get_module_namespace()))
}

#[op2]
pub fn op_loader_module_requests<'s>(
  scope: &mut v8::HandleScope<'s>,
  #[smi] module_id: usize
) -> Result<v8::Local<'s, v8::Value>, ModuleLoaderError> {
  let map = unsafe { JsRuntime::module_map_from(scope) };
  let requests = match map.get_requested_modules(module_id) {
    Some(requested_modules) => requested_modules,
    None => return Err(ModuleLoaderError::NotFound)
  };

  let requests = requests
      .into_iter()
      .map(|request| request.specifier)
      .map(|specifier| v8::String::new(scope, specifier.as_str()).unwrap())
      .map(|string| string.cast())
      .collect::<Vec<_>>();

  Ok(v8::Array::new_with_elements(scope, &requests).cast())
}

#[op2]
pub fn op_loader_resolve<'s>(
  scope: &mut v8::HandleScope<'s>,
  #[varargs] args: Option<&v8::FunctionCallbackArguments>,
  #[string] specifier: &str
) -> Result<v8::Local<'s, v8::Value>, ModuleLoaderError> {
  let map = unsafe { JsRuntime::module_map_from(scope) };
  let referrer = args.and_then(|args| {
    let data = args.data();
    if data.is_string() { Some(data.to_rust_string_lossy(scope)) }
    else { None }
  }).unwrap_or(String::new());

  let resolved = map.resolve(specifier, &referrer, ResolutionKind::DynamicImport)?;

  Ok(v8::String::new(scope, resolved.as_str()).unwrap().cast())
}
