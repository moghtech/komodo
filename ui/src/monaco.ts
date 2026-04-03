import * as monaco from "monaco-editor";

/**
 * Don't need to await to continue app render,
 * will apply in background when ready.
 */
export default async function initMonaco() {
  const promises = ["lib", "responses", "types", "terminal"].map((file) =>
    Promise.all(
      [".js", ".d.ts"].map((extension) =>
        fetch(`/client/${file}${extension}`)
          .then((res) => res.text())
          .then((dts) =>
            monaco.typescript.typescriptDefaults.addExtraLib(
              dts,
              `file:///client/${file}${extension}`,
            ),
          ),
      ),
    ),
  );
  promises.push(
    Promise.all(
      ["index.d.ts", "deno.d.ts"].map((file) =>
        fetch(`/${file}`)
          .then((res) => res.text())
          .then((dts) =>
            monaco.typescript.typescriptDefaults.addExtraLib(
              dts,
              `file:///${file}`,
            ),
          ),
      ),
    ),
  );

  await Promise.all(promises);

  monaco.typescript.typescriptDefaults.setCompilerOptions({
    module: monaco.typescript.ModuleKind.ESNext,
    target: monaco.typescript.ScriptTarget.ESNext,
    allowNonTsExtensions: true,
    moduleResolution: monaco.typescript.ModuleResolutionKind.NodeJs,
    typeRoots: ["index.d.ts"],
    allowTopLevelAwait: true,
    moduleDetection: 1,
  } as monaco.typescript.CompilerOptions);

  monaco.typescript.typescriptDefaults.setDiagnosticsOptions({
    diagnosticCodesToIgnore: [
      // Allows top level await
      1375,
      // Allows top level return
      1108,
    ],
  });
}
