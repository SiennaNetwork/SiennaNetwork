import HtmlWebpackPlugin from 'html-webpack-plugin'
export default 
{ mode:      "development"
, entry:     "./rewards_dashboard.ts"
, module:    { rules:
  [ { test: /\.tsx?$/, use: 'ts-loader', exclude: /node_modules/ }
  , { test: /\.css$/i, use: ["style-loader", "css-loader"], }
  , /*{ test: /\.wasm$/i, use: ["wasm-loader"], },*/
  ] }
, plugins:   [ new HtmlWebpackPlugin () ]
, resolve:   { extensions: ['.tsx', '.ts', '.js', /*'.wasm'*/ ] }
, devtool:   'inline-source-map'
, devServer: { contentBase: './target/web' } }
