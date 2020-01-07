const path = require('path');
const PrerenderSPAPlugin = require('prerender-spa-plugin');
const BundleAnalyzerPlugin = require('webpack-bundle-analyzer').BundleAnalyzerPlugin;
const webpack = require('webpack');

module.exports = {
  "transpileDependencies": [
    "vuetify"
  ],
  configureWebpack: {
    plugins: [
      new PrerenderSPAPlugin({
        staticDir: path.join(__dirname, 'dist'),
        routes: ['/']
      }),
      // ignore locales from moment to reduce size
      new webpack.IgnorePlugin(/^\.\/locale$/, /moment$/),

      // enable for bundle size analysis
      // new BundleAnalyzerPlugin(),
    ]
  }
}
