import * as monaco from "monaco-editor";

export async function init_monaco() {
  const promises = [
    "lib",
    "responses",
    "types",
    "types_namespace",
    "terminal",
  ].map((file) =>
    Promise.all(
      [".js", ".d.ts"].map((extension) =>
        fetch(`/client/${file}${extension}`)
          .then((res) => res.text())
          .then((dts) =>
            monaco.languages.typescript.typescriptDefaults.addExtraLib(
              dts,
              `file:///client/${file}${extension}`
            )
          )
      )
    )
  );
  promises.push(
    Promise.all(
      ["index.d.ts", "deno.d.ts"].map((file) =>
        fetch(`/${file}`)
          .then((res) => res.text())
          .then((dts) =>
            monaco.languages.typescript.typescriptDefaults.addExtraLib(
              dts,
              `file:///${file}`
            )
          )
      )
    )
  );

  await Promise.all(promises);

  type ExtraOptions = {
    allowTopLevelAwait?: boolean;
    moduleDetection?: "force" | "auto" | "legacy" | 3 | 2 | 1; // string or numeric enum
  };

  monaco.languages.typescript.typescriptDefaults.setCompilerOptions({
    module: monaco.languages.typescript.ModuleKind.ESNext,
    target: monaco.languages.typescript.ScriptTarget.ESNext,
    allowNonTsExtensions: true,
    moduleResolution: monaco.languages.typescript.ModuleResolutionKind.NodeJs,
    typeRoots: ["index.d.ts"],
    allowTopLevelAwait: true,
    moduleDetection: "force",
  } as monaco.languages.typescript.CompilerOptions & ExtraOptions);

  monaco.languages.typescript.typescriptDefaults.setDiagnosticsOptions({
    diagnosticCodesToIgnore: [
      // Allows top level await
      1375,
      // Allows top level return
      1108,
    ],
  });
}
