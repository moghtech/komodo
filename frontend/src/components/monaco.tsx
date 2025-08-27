import { useEffect, useState } from "react";
import { DiffEditor, Editor } from "@monaco-editor/react";
import { useTheme } from "@ui/theme";
import { cn } from "@lib/utils";
import * as monaco from "monaco-editor";
import * as prettier from "prettier/standalone";
import * as pluginTypescript from "prettier/plugins/typescript";
import * as pluginEsTree from "prettier/plugins/estree";
import * as pluginYaml from "prettier/plugins/yaml";
import { useRead, useWindowDimensions } from "@lib/hooks";

const MIN_EDITOR_HEIGHT = 56;

export type MonacoLanguage =
  | "yaml"
  | "toml"
  | "fancy_toml"
  | "json"
  | "key_value"
  | "ini"
  | "string_list"
  | "shell"
  | "dockerfile"
  | "rust"
  | "javascript"
  | "typescript";

const LANGUAGE_EXTENSIONS: Record<MonacoLanguage, string[]> = {
  yaml: [".yaml", ".yml"],
  toml: [".toml"],
  fancy_toml: [],
  json: [".json"],
  key_value: [".env", ".conf"],
  ini: [".ini"],
  string_list: [],
  shell: [".sh", ".bash", ".zsh"],
  dockerfile: ["Dockerfile"],
  rust: [".rs"],
  javascript: [".js", ".jsx", ".mjs", ".cjs"],
  typescript: [".ts", ".tsx"],
};

export const language_from_path = (path: string) => {
  for (const [lang, extensions] of Object.entries(LANGUAGE_EXTENSIONS)) {
    for (const extension of extensions) {
      if (path.endsWith(extension)) {
        return lang as MonacoLanguage;
      }
    }
  }
  return undefined;
};

export const MonacoEditor = ({
  value,
  onValueChange,
  language: _language,
  readOnly,
  filename,
  minHeight,
  className,
}: {
  value: string | undefined;
  onValueChange?: (value: string) => void;
  language: MonacoLanguage | undefined;
  filename?: string;
  readOnly?: boolean;
  minHeight?: number;
  className?: string;
}) => {
  const enable_fancy_toml =
    useRead("GetCoreInfo", {}).data?.enable_fancy_toml ?? false;
  const language = (
    _language === "fancy_toml" && !enable_fancy_toml ? "toml" : _language
  ) as MonacoLanguage;
  const dimensions = useWindowDimensions();
  const [editor, setEditor] =
    useState<monaco.editor.IStandaloneCodeEditor | null>(null);

  useEffect(() => {
    if (!editor) return;

    let node = editor.getDomNode();
    if (!node) return;

    const callback = (e: any) => {
      if (e.key === "Escape") {
        (document.activeElement as any)?.blur?.();
      }
    };

    node.addEventListener("keydown", callback);
    return () => node.removeEventListener("keydown", callback);
  }, [editor]);

  useEffect(() => {
    if (
      language !== "typescript" &&
      language !== "javascript" &&
      language !== "yaml"
    )
      return;
    if (!editor) return;
    editor.addCommand(
      monaco.KeyMod.Alt | monaco.KeyMod.Shift | monaco.KeyCode.KeyF,
      async () => {
        if (!editor) return;
        const model = editor.getModel();
        if (!model) return;
        const position = editor.getPosition();
        let beforeOffset = (position && model.getOffsetAt(position)) ?? 0;
        const curr = editor.getValue();
        const { formatted, cursorOffset } = await prettier.formatWithCursor(
          curr,
          {
            cursorOffset: beforeOffset,
            parser: language === "yaml" ? "yaml" : "typescript",
            plugins:
              language === "yaml"
                ? [pluginYaml]
                : [pluginTypescript, pluginEsTree as any],
            printWidth: 80, // Set the desired max line length
          }
        );
        editor.setValue(formatted);
        editor.setPosition(model.getPositionAt(cursorOffset));
      }
    );
  }, [editor]);

  const line_count = value?.split(/\r\n|\r|\n/).length ?? 0;

  useEffect(() => {
    if (!editor) return;
    const contentHeight = line_count * 18 + 30;
    const containerNode = editor.getContainerDomNode();

    containerNode.style.height = `${Math.max(
      Math.min(contentHeight, Math.floor(dimensions.height * 0.5)),
      minHeight ?? MIN_EDITOR_HEIGHT
    )}px`;
  }, [editor, line_count]);

  const { theme: _theme } = useTheme();
  const theme =
    _theme === "system"
      ? window.matchMedia("(prefers-color-scheme: dark)").matches
        ? "dark"
        : "light"
      : _theme;

  const options: monaco.editor.IStandaloneEditorConstructionOptions = {
    minimap: { enabled: false },
    // scrollbar: { alwaysConsumeMouseWheel: false },
    scrollBeyondLastLine: false,
    folding: false,
    automaticLayout: true,
    renderValidationDecorations: "on",
    renderLineHighlightOnlyWhenFocus: true,
    readOnly,
    tabSize: 2,
    detectIndentation: true,
    quickSuggestions: true,
    padding: {
      top: 15,
    },
  };

  return (
    <div className={cn("mx-2 my-1 w-full", className)}>
      <Editor
        language={language}
        value={value}
        theme={theme}
        defaultPath={filename ? `file:///${filename}` : undefined}
        options={options}
        onChange={(v) => onValueChange?.(v ?? "")}
        onMount={(editor) => setEditor(editor)}
      />
    </div>
  );
};

const MIN_DIFF_HEIGHT = 100;
const MAX_DIFF_HEIGHT = 400;

export const MonacoDiffEditor = ({
  original,
  modified,
  onModifiedValueChange,
  language: _language,
  readOnly,
  containerClassName,
  hideUnchangedRegions = true,
}: {
  original: string | undefined;
  modified: string | undefined;
  onModifiedValueChange?: (value: string) => void;
  language: MonacoLanguage | undefined;
  readOnly?: boolean;
  containerClassName?: string;
  hideUnchangedRegions?: boolean;
}) => {
  const enable_fancy_toml =
    useRead("GetCoreInfo", {}).data?.enable_fancy_toml ?? false;
  const language = (
    _language === "fancy_toml" && !enable_fancy_toml ? "toml" : _language
  ) as MonacoLanguage;

  const [editor, setEditor] =
    useState<monaco.editor.IStandaloneDiffEditor | null>(null);

  const original_line_count = original?.split(/\r\n|\r|\n/).length ?? 0;
  const modified_line_count = modified?.split(/\r\n|\r|\n/).length ?? 0;
  const line_count = Math.max(original_line_count, modified_line_count);

  useEffect(() => {
    if (!editor) return;
    const contentHeight = line_count * 18 + 30;
    const node = editor.getContainerDomNode();

    node.style.height = `${Math.max(
      Math.min(contentHeight, MAX_DIFF_HEIGHT),
      MIN_DIFF_HEIGHT
    )}px`;
  }, [editor, line_count]);

  const { theme: _theme } = useTheme();
  const theme =
    _theme === "system"
      ? window.matchMedia("(prefers-color-scheme: dark)").matches
        ? "dark"
        : "light"
      : _theme;

  const options: monaco.editor.IStandaloneDiffEditorConstructionOptions = {
    minimap: { enabled: true },
    scrollbar: { alwaysConsumeMouseWheel: false },
    scrollBeyondLastLine: false,
    hideUnchangedRegions: { enabled: hideUnchangedRegions },
    folding: false,
    automaticLayout: true,
    renderValidationDecorations: "on",
    renderLineHighlightOnlyWhenFocus: true,
    readOnly,
    padding: {
      top: 15,
    },
  };

  return (
    <div className={cn("mx-2 my-1", containerClassName)}>
      <DiffEditor
        language={language}
        original={original}
        modified={modified}
        theme={theme}
        options={options}
        onMount={(editor) => {
          const modifiedEditor = editor.getModifiedEditor();
          modifiedEditor.onDidChangeModelContent((_) => {
            onModifiedValueChange?.(modifiedEditor.getValue());
          });
          setEditor(editor);
        }}
      />
    </div>
  );
};
