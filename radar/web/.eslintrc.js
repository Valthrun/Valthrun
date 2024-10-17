module.exports = {
    root: true,
    parser: "@typescript-eslint/parser",
    parserOptions: {
        project: "tsconfig.json",
        sourceType: "module",
    },
    plugins: ["@typescript-eslint/eslint-plugin"],
    extends: ["plugin:@typescript-eslint/recommended", "plugin:prettier/recommended"],
    env: {
        node: true,
        jest: true,
    },
    ignorePatterns: [".eslintrc.js", "**/dist/", "**/node_modules/", "src/backend/definitions.ts"],
    rules: {
        "prettier/prettier": 0,
        "@typescript-eslint/interface-name-prefix": "off",
        "@typescript-eslint/explicit-function-return-type": "off",
        "@typescript-eslint/explicit-module-boundary-types": "off",
        "@typescript-eslint/no-explicit-any": "off",
        "@typescript-eslint/no-non-null-assertion": "off",
        "@typescript-eslint/no-unused-vars": [
            "error",
            {
                argsIgnorePattern: "^_",
                varsIgnorePattern: "^_",
                caughtErrorsIgnorePattern: "^_",
            },
        ],
    },
};
