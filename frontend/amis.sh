dist=public/static/lib/jssdk
fontawesome_dist=$dist/thirds/@fortawesome/fontawesome-free/webfonts
monaco_dist=$dist/thirds/monaco-editor/min/vs
sdk=node_modules/amis/sdk

rm -rf $dist
mkdir -p $fontawesome_dist
mkdir -p $monaco_dist/editor
mkdir -p $monaco_dist/language/json

cp $sdk/antd.css $dist
cp $sdk/iconfont.css $dist
cp $sdk/iconfont.eot $dist
cp $sdk/iconfont.svg $dist
cp $sdk/iconfont.ttf $dist
cp $sdk/iconfont.woff $dist

cp $sdk/sdk.js $dist
cp $sdk/rest.js $dist
cp $sdk/cropperjs.js $dist

cp $sdk/thirds/@fortawesome/fontawesome-free/webfonts/fa-solid-900.ttf $fontawesome_dist
cp $sdk/thirds/@fortawesome/fontawesome-free/webfonts/fa-solid-900.woff2 $fontawesome_dist
cp $sdk/thirds/monaco-editor/min/vs/loader.js $monaco_dist
cp $sdk/thirds/monaco-editor/min/vs/editor/editor.main.js $monaco_dist/editor
cp $sdk/thirds/monaco-editor/min/vs/editor/editor.main.nls.js $monaco_dist/editor
cp $sdk/thirds/monaco-editor/min/vs/language/json/jsonMode.js $monaco_dist/language/json
