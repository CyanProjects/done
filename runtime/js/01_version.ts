// Copyright 2018-2025 the Deno authors. MIT license.

import { primordials } from "ext:core/mod.js";
const {
  ObjectFreeze,
} = primordials;

interface Version {
  deno: string;
  v8: string;
  typescript: string;
  done: string;
}

const version: Version = {
  deno: "",
  v8: "",
  typescript: "",
  done: ""
};

function setVersions(
  denoVersion,
  doneCode,
  v8Version,
  tsVersion,
) {
  version.deno = denoVersion;
  version.v8 = v8Version;
  version.typescript = tsVersion;
  version.done = doneCode;

  ObjectFreeze(version);
}

export { setVersions, version };
