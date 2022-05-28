const publicPath = "/amm";

module.exports = {
  publicPath: publicPath,
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
