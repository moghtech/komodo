import * as monaco from "monaco-editor";

/// V2: Toml + Yaml + Env Vars
const fancy_toml_conf: monaco.languages.LanguageConfiguration = {
  comments: {
    lineComment: "#",
  },
  brackets: [
    ["{", "}"],
    ["[", "]"],
    ["(", ")"],
  ],
  autoClosingPairs: [
    { open: "{", close: "}" },
    { open: "[", close: "]" },
    { open: "(", close: ")" },
    { open: '"', close: '"' },
    { open: "'", close: "'" },
    { open: '"""', close: '"""' },
  ],
  surroundingPairs: [
    { open: "{", close: "}" },
    { open: "[", close: "]" },
    { open: "(", close: ")" },
    { open: '"', close: '"' },
    { open: "'", close: "'" },
    { open: '"""', close: '"""' },
  ],
};

const fancy_toml_language = <monaco.languages.IMonarchLanguage>{
  defaultToken: "",
  tokenPostfix: ".toml",

  escapes: /\\(?:[btnfr\"\'\\\/]|u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{8})/,

  tokenizer: {
    root: [
      // Comments
      [/\s*((#).*)$/, "comment"],

      // Table Definitions
      [
        /^\s*(\[\[)([^[\]]+)(\]\])/,
        [
          "punctuation.definition.array.table",
          "entity.other.attribute-name.table.array",
          "punctuation.definition.array.table",
        ],
      ],
      [
        /^\s*(\[)([^[\]]+)(\])/,
        [
          "punctuation.definition.table",
          "entity.other.attribute-name.table",
          "punctuation.definition.table",
        ],
      ],

      // Inline tables
      [
        /\{/,
        {
          token: "punctuation.definition.table.inline",
          next: "@inlineTable",
        },
      ],

      // Entry (Key = Value)
      [
        /\s*((?:(?:(?:[A-Za-z0-9_+-]+)|(?:\"[^\"]+\")|(?:'[^']+'))\s*\.?\s*)+)\s*(=)/,
        ["", "delimiter"],
      ],

      // Values (booleans, numbers, dates, strings, arrays)
      { include: "@values" },
    ],

    // Inline Table
    inlineTable: [
      [/\}/, { token: "punctuation.definition.table.inline", next: "@pop" }],
      { include: "@comments" },
      [/,/, "punctuation.separator.table.inline"],
      { include: "@values" },
    ],

    // Values (Strings, Numbers, Booleans, Dates, Arrays)
    values: [
      // Triple quoted string (basic)
      [/"""/, { token: "string", next: "@tripleStringWithYamlEnv" }],

      // Single quoted string
      [/"/, { token: "string.quoted.single.basic.line", next: "@basicString" }],

      // Triple quoted literal string
      [
        /'''/,
        {
          token: "string.quoted.triple.literal.block",
          next: "@literalTripleStringWithYamlEnv",
        },
      ],

      // Single quoted literal string
      [
        /'/,
        {
          token: "string.quoted.single.literal.line",
          next: "@literalStringSingle",
        },
      ],

      // Dates and Times
      [
        /\d{4}-\d{2}-\d{2}[Tt ]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})/,
        "constant.other.time.datetime.offset",
      ],
      [
        /\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?/,
        "constant.other.time.datetime.local",
      ],
      [/\d{4}-\d{2}-\d{2}/, "constant.other.time.date"],
      [/\d{2}:\d{2}:\d{2}(?:\.\d+)?/, "constant.other.time.time"],

      // Booleans
      [/\b(true|false)\b/, "constant.language.boolean"],

      // Numbers
      [/[+-]?(0x[0-9A-Fa-f_]+|0o[0-7_]+|0b[01_]+)/, "number.hex"],
      [
        /(?<!\w)([+-]?(0|([1-9](([0-9]|_[0-9])+)?))(?:(?:\.(0|([1-9](([0-9]|_[0-9])+)?)))?[eE][+-]?[1-9]_?[0-9]*|(?:\.[0-9_]*)))(?!\w)/,
        "number.float",
      ],
      [/(?<!\w)((?:[+-]?(0|([1-9](([0-9]|_[0-9])+)?))))(?!\w)/, "number"],

      // Arrays
      [/\[/, { token: "punctuation.definition.array", next: "@array" }],
    ],

    // Basic quoted string
    basicString: [
      [/[^\\"]+/, "string"],
      [/@escapes/, "constant.character.escape"],
      [/\\./, "invalid"],
      [/"/, { token: "string.quoted.single.basic.line", next: "@pop" }],
    ],

    // Literal triple quoted string
    literalStringTriple: [
      [/[^']+/, "string"],
      [/'/, { token: "string.quoted.triple.literal.block", next: "@pop" }],
    ],

    // Literal single quoted string
    literalStringSingle: [
      [/[^']+/, "string"],
      [/'/, { token: "string.quoted.single.literal.line", next: "@pop" }],
    ],

    // Arrays
    array: [
      [/\]/, { token: "punctuation.definition.array", next: "@pop" }],
      [/,/, "punctuation.separator.array"],
      { include: "@values" },
    ],

    // Handle whitespace and comments
    whitespace: [[/\s+/, ""]],
    comments: [[/\s*((#).*)$/, "comment.line.number-sign"]],

    // CUSTOM STUFF FOR YAML / ENV IN TRIPLE STRING

    tripleStringWithYamlEnv: [
      [/"""/, { token: "string", next: "@pop" }],
      { include: "@yamlTokenizer" }, // YAML inside triple quotes
      { include: "@envVariableTokenizer" }, // Environment Variable parsing inside triple quotes
    ],

    literalTripleStringWithYamlEnv: [
      [/'''/, { token: "string", next: "@pop" }],
      { include: "@yamlTokenizer" }, // YAML inside triple quotes
      { include: "@envVariableTokenizer" }, // Environment Variable parsing inside triple quotes
    ],

    // YAML Tokenizer for inside triple quotes
    yamlTokenizer: [
      { include: "@yaml_whitespace" },
      { include: "@yaml_comments" },
      { include: "@yaml_keys" },
      { include: "@yaml_numbers" },
      { include: "@yaml_booleans" },
      { include: "@yaml_strings" },
      { include: "@yaml_constants" },
    ],

    // Environment Variable Tokenizer
    envVariableTokenizer: [
      [
        /(\s*-*\s*)([A-Za-z0-9_]+)(\s*)(=|:)(\s*)/,
        ["", "key", "", "operator.assignment", ""],
      ],
      { include: "@yamlTokenizer" }, // Use YAML tokenizer for EnvVar values
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
      [/"([^"\\]|\\.)*$/, "string.invalid"], // Non-terminated string
      [/'([^'\\]|\\.)*$/, "string.invalid"], // Non-terminated string
      [/"/, "string", "@yaml_string_double"],
      [/'/, "string", "@yaml_string_single"],
    ],
    yaml_string_double: [
      [/[^\\"]+/, "string"],
      [/@escapes/, "string.escape"],
      [/\\./, "string.escape.invalid"],
      [/"/, { token: "string", next: "@pop" }],
    ],
    yaml_string_single: [
      [/[^\\']+/, "string"],
      [/@escapes/, "string.escape"],
      [/\\./, "string.escape.invalid"],
      [/'/, { token: "string", next: "@pop" }],
    ],
    yaml_constants: [[/\b(null|~)\b/, "constant.language.null"]],
  },
};

monaco.languages.register({ id: "fancy_toml" });
monaco.languages.setLanguageConfiguration("fancy_toml", fancy_toml_conf);
monaco.languages.setMonarchTokensProvider("fancy_toml", fancy_toml_language);
