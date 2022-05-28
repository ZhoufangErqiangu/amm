const publicPath = "/";

module.exports = {
  publicPath: publicPath,
  devServer: {
    open: true,
    overlay: {
      warnings: false,
      errors: true,
    },
  },
  css: {
    extract: true,
    sourceMap: false,
  },
};
