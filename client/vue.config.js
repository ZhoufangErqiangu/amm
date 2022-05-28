const publicPath = "/amm/";

module.exports = {
  publicPath: process.env.ENV !== "production" ? "/" : publicPath,
  productionSourceMap: false,
  devServer: {
    open: true,
    overlay: {
      warnings: false,
      errors: true,
    },
  },
  css: {
    extract: true,
    sourceMap: process.env.ENV !== "production",
  },
};
