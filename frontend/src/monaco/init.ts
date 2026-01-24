import * as monaco from "monaco-editor";

export async function init_monaco() {
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

  type ExtraOptions = {
    allowTopLevelAwait?: boolean;
    moduleDetection?: "force" | "auto" | "legacy" | 3 | 2 | 1; // string or numeric enum
  };

  monaco.typescript.typescriptDefaults.setCompilerOptions({
    module: monaco.typescript.ModuleKind.ESNext,
    target: monaco.typescript.ScriptTarget.ESNext,
    allowNonTsExtensions: true,
    moduleResolution: monaco.typescript.ModuleResolutionKind.NodeJs,
    typeRoots: ["index.d.ts"],
    allowTopLevelAwait: true,
    moduleDetection: "force",
  } as monaco.typescript.CompilerOptions & Partial<ExtraOptions>);

  monaco.typescript.typescriptDefaults.setDiagnosticsOptions({
    diagnosticCodesToIgnore: [
      // Allows top level await
      1375,
      // Allows top level return
      1108,
    ],
  });
}
