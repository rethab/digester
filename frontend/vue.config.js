const path = require('path');
const PrerenderSPAPlugin = require('prerender-spa-plugin');
const BundleAnalyzerPlugin = require('webpack-bundle-analyzer').BundleAnalyzerPlugin;
const webpack = require('webpack');

module.exports = {
  "transpileDependencies": [
    "vuetify"
  ],
  devServer: {
    https: true
  },
  configureWebpack: {
    devServer: {
      disableHostCheck: true,
      headers: {
        'Content-Security-Policy': "default-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'; img-src 'self'; block-all-mixed-content; font-src https://cdn.jsdelivr.net; style-src 'self' 'nonce-dc77ae858e' https://cdn.jsdelivr.net; connect-src https://api-stg.digester.app; script-src 'self';"
      }
    },
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
