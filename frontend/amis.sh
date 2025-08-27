dist=public/static/lib/jssdk
fontawesome_dist=$dist/thirds/@fortawesome/fontawesome-free/webfonts
sdk=node_modules/amis/sdk

rm -rf $dist
mkdir -p $fontawesome_dist

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
