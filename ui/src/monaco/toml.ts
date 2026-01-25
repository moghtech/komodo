import * as monaco from "monaco-editor";

/* -------------------------------------------------
 *  Language configuration  (unchanged)
 * ------------------------------------------------- */
const toml_conf: monaco.languages.LanguageConfiguration = {
  comments: { lineComment: "#" },
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

/* -------------------------------------------------
 *  Monarch tokenizer – TOML-only
 * ------------------------------------------------- */
const toml_language: monaco.languages.IMonarchLanguage = {
  defaultToken: "",
  tokenPostfix: ".toml",

  escapes: /\\(?:[btnfr"'\\\/]|u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{8})/,

  tokenizer: {
    /* ---------- root ---------- */
    root: [
      { include: "@comments" },

      /* Tables & array-tables */
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

      /* Inline tables */
      [
        /\{/,
        { token: "punctuation.definition.table.inline", next: "@inlineTable" },
      ],

      /* Key-value pair */
      [
        /\s*((?:(?:(?:[A-Za-z0-9_+\-]+)|(?:\"[^\"]+\")|(?:'[^']+'))\s*\.?\s*)+)\s*(=)/,
        ["", "delimiter"],
      ],

      /* Values */
      { include: "@values" },
    ],

    /* ---------- inline table ---------- */
    inlineTable: [
      [/\}/, { token: "punctuation.definition.table.inline", next: "@pop" }],
      { include: "@comments" },
      [/,/, "punctuation.separator.table.inline"],
      { include: "@values" },
    ],

    /* ---------- values ---------- */
    values: [
      /* Strings ---------------------------------------------------- */
      [/"""/, { token: "string", next: "@tripleBasicString" }],
      [/"/, { token: "string", next: "@basicString" }],
      [/'''/, { token: "string", next: "@tripleLiteralString" }],
      [/'/, { token: "string", next: "@literalStringSingle" }],

      /* Dates, times, booleans ------------------------------------ */
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
      [/\b(true|false)\b/, "constant.language.boolean"],

      /* Numbers ---------------------------------------------------- */
      [/[+-]?(0x[0-9A-Fa-f_]+|0o[0-7_]+|0b[01_]+)/, "number.hex"],
      [
        /[+-]?(?:\d(?:_?\d)*)(?:\.\d(?:_?\d)*)?(?:[eE][+-]?\d(?:_?\d)*)?/,
        "number.float",
      ],
      [/[+-]?\d(?:_?\d)*/, "number"],

      /* Arrays ----------------------------------------------------- */
      [/\[/, { token: "punctuation.definition.array", next: "@array" }],
    ],

    /* ---------- arrays ---------- */
    array: [
      [/\]/, { token: "punctuation.definition.array", next: "@pop" }],
      [/,/, "punctuation.separator.array"],
      { include: "@values" },
    ],

    /* ---------- strings ---------- */
    basicString: [
      [/[^\\"]+/, "string"],
      [/@escapes/, "string.escape"],
      [/\\./, "invalid"],
      [/"/, { token: "string", next: "@pop" }],
    ],

    tripleBasicString: [
      [/"""/, { token: "string", next: "@pop" }],
      [/[^\\"]+/, "string"],
      [/@escapes/, "string.escape"],
      [/\\./, "string.invalid"],
    ],

    literalStringSingle: [
      [/[^']+/, "string"],
      [/'/, { token: "string", next: "@pop" }],
    ],

    tripleLiteralString: [
      [/'''/, { token: "string", next: "@pop" }],
      [/[^']+/, "string"],
    ],

    /* ---------- misc helpers ---------- */
    comments: [[/\s*((#).*)$/, "comment"]],
  },
};

/* -------------------------------------------------
 *  Register with Monaco
 * ------------------------------------------------- */
monaco.languages.register({ id: "toml" });
monaco.languages.setLanguageConfiguration("toml", toml_conf);
monaco.languages.setMonarchTokensProvider("toml", toml_language);
