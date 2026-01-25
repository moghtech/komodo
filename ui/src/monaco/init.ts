import * as monaco from "monaco-editor";

export async function init_monaco() {
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
