import * as monaco from "monaco-editor";

// Language Configuration
const key_value_conf: monaco.languages.LanguageConfiguration = {
  comments: {
    lineComment: "#",
  },
  brackets: [],
  autoClosingPairs: [
    { open: '"', close: '"' },
    { open: "'", close: "'" },
  ],
  surroundingPairs: [
    { open: '"', close: '"' },
    { open: "'", close: "'" },
  ],
};

// Language Definition (Monarch Tokenizer)
const key_value_language = <monaco.languages.IMonarchLanguage>{
  defaultToken: "",
  tokenPostfix: ".env",

  escapes:
    /\\(?:[abfnrtv\\"']|x[0-9A-Fa-f]{1,4}|u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{8})/,

  tokenizer: {
    root: [
      // Handle environment variables (KEY = VALUE or KEY: VALUE)
      [
        /(\s*-*\s*)([A-Za-z0-9_]+)(\s*)(=|:)(\s*)/,
        [
          "", // Optional leading hyphen
          "key", // Key (environment variable)
          "", // Whitespace
          "operator.assignment", // Equals sign (=) or colon (:)
          "", // Whitespace
        ],
      ],

      // Parse value as yaml
      { include: "@yaml_whitespace" },
      { include: "@yaml_comments" },
      { include: "@yaml_keys" },
      { include: "@yaml_numbers" },
      { include: "@yaml_booleans" },
      { include: "@yaml_strings" },
      { include: "@yaml_constants" },
    ],

    yaml_whitespace: [[/[ \t\r\n]+/, ""]],

    yaml_comments: [[/#.*$/, "comment"]],

    yaml_keys: [[/([^\s\[\]{},"']+)(\s*)(:)/, ["key", "", "delimiter"]]],

    yaml_numbers: [
      [/\b\d+\.\d*\b/, "number.float"],
      [/\b0x[0-9a-fA-F]+\b/, "number.hex"],
      [/\b\d+\b/, "number"],
    ],

    yaml_booleans: [
      [/\b(true|false|yes|no|on|off)\b/, "constant.language.boolean"],
    ],

    yaml_strings: [
      [/"([^"\\]|\\.)*$/, "string.invalid"], // non-terminated string
      [/'([^'\\]|\\.)*$/, "string.invalid"], // non-terminated string
      [/"/, "string", "@yaml_string_double"],
      [/'/, "string", "@yaml_string_single"],
    ],

    yaml_string_double: [
      [/[^\\"]+/, "string"],
      [/@escapes/, "string.escape"],
      [/\\./, "string.escape.invalid"],
      [/"/, "string", "@pop"],
    ],

    yaml_string_single: [
      [/[^\\']+/, "string"],
      [/@escapes/, "string.escape"],
      [/\\./, "string.escape.invalid"],
      [/'/, "string", "@pop"],
    ],

    yaml_constants: [[/\b(null|~)\b/, "constant.language.null"]],
  },
};

// Register the new language
monaco.languages.register({ id: "key_value" });

// Set the language configuration and tokenizer
monaco.languages.setLanguageConfiguration("key_value", key_value_conf);
monaco.languages.setMonarchTokensProvider("key_value", key_value_language);
