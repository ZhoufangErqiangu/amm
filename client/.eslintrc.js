module.exports = {
  root: true,
  env: {
    node: true,
  },
  plugins: ["prettier"],
  extends: ["plugin:vue/strongly-recommended", "prettier"],
  parser: "vue-eslint-parser",
  parserOptions: {
    parser: "babel-eslint",
  },
  rules: {
    "no-console": "off",
    "no-await-in-loop": "off",
    // 'vue/singleline-html-element-content-newline': 'off',
  },
};
