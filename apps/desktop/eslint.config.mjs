import eslint from "@eslint/js";
import globals from "globals";
import tseslint from "typescript-eslint";
import vue from "eslint-plugin-vue";

export default tseslint.config(
  {
    ignores: [
      "dist/",
      "coverage/",
      "playwright-report/",
      "test-results/",
      "target/",
      "src/types/generated/",
      "**/*.vue.js",
      "**/*.vue.d.ts",
      "**/*.tsbuildinfo",
    ],
  },
  eslint.configs.recommended,
  ...tseslint.configs.recommended,
  ...vue.configs["flat/recommended"],
  {
    files: ["scripts/**/*.mjs", "*.mjs"],
    languageOptions: {
      globals: globals.node,
    },
  },
  {
    files: ["src/**/*.{ts,vue}"],
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
    rules: {
      "vue/multi-word-component-names": "off",
      "vue/attributes-order": "off",
      "vue/html-closing-bracket-newline": "off",
      "vue/html-indent": "off",
      "vue/html-self-closing": "off",
      "vue/max-attributes-per-line": "off",
      "vue/multiline-html-element-content-newline": "off",
      "vue/no-restricted-html-elements": [
        "error",
        {
          element: ["button", "dialog", "input", "select", "textarea"],
          message:
            "Use the app-owned PrimeVue wrapper from src/components/ui instead.",
        },
      ],
      "vue/singleline-html-element-content-newline": "off",
    },
  },
  {
    files: ["src/**/*.vue"],
    languageOptions: {
      parserOptions: {
        parser: tseslint.parser,
      },
    },
  },
);
