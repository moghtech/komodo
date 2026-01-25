import * as monaco from "monaco-editor";

const string_list_conf: monaco.languages.LanguageConfiguration = {
  comments: {
    lineComment: "#",
  },
  autoClosingPairs: [
    { open: '"', close: '"' },
    { open: "'", close: "'" },
  ],
  surroundingPairs: [
    { open: '"', close: '"' },
    { open: "'", close: "'" },
  ],
};

const string_list_language = <monaco.languages.IMonarchLanguage>{
  defaultToken: "",
  tokenPostfix: ".string_list",

  tokenizer: {
    root: [
      // Comments
      [/#.*$/, "comment"],

      // Comma as a delimiter
      [/,/, "comment"],
      [/\*/, "keyword"],
      [/\?/, "keyword"],

      // Special syntax: text surrounded by \
      // [/\\[^\\]*\\/, "keyword"],
      [/\\/, { token: "keyword", next: "@regex" }],

      // Main strings separated by spaces or newlines
      [/[^\*\?,#\\\s]+/, ""],

      // Whitespace
      [/[ \t\r\n]+/, ""],
    ],
    regex: [
      // Regex tokens
      [/\[[^\]]*\]/, ""], // Character classes like [abc]
      [/[*+?\.]+/, "keyword"], // Quantifiers like *, +, ?
      [/\\./, "string.regexp constant.character.escape"], // Escape sequences like \d, \w
      [/[^\\]/, "string"], // Any other regex content
      [/\\/, { token: "keyword", next: "@pop" }], // Closing backslash returns to root
    ],
  },
};

// Register the custom language and configuration with Monaco
monaco.languages.register({ id: "string_list" });
monaco.languages.setLanguageConfiguration("string_list", string_list_conf);
monaco.languages.setMonarchTokensProvider("string_list", string_list_language);
