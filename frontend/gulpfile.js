import { src, dest, series } from 'gulp';
import gulpGzip from 'gulp-gzip';
import gulpif from 'gulp-if';
import del from 'delete';
const destPath = './dist';
const clean = (cb) => {
    del([destPath], cb);
};

const gzipify = () => {
    return src('./dist_rsbuild/**/*', {
        encoding: false
    })
        .pipe(gulpif(file => {
            return [".json", ".html", ".mjs", ".js", ".css", ".svg", ".ttf", ".woff", ".woff2", ".eot"].includes(file.extname);
        }, gulpGzip()))
        .pipe(dest(destPath));
};

export default series([clean, gzipify]);