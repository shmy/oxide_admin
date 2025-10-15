import { src, dest, series, task } from 'gulp';
import gulpGzip from 'gulp-gzip';
import gulpBrotli from 'gulp-brotli';
import gulpif from 'gulp-if';
import del from 'delete';
import fs from 'node:fs';
import path from 'node:path';

const destPath = './dist';
const includeExtensions = ['.json', '.mjs', '.js', '.css', '.svg', '.ttf', '.woff', '.woff2', '.eot'];

const clean = (done) => {
    del([destPath], done);
};

const gzipify = (done) => {
    return src('./dist_rsbuild/**/*', { encoding: false })
        .pipe(gulpif(file => includeExtensions.includes(file.extname.toLowerCase()), gulpGzip()))
        .pipe(dest(destPath))
        .on("finish", done);
};

const brotlify = (done) => {
    return src('./dist_rsbuild/**/*', { encoding: false })
        .pipe(gulpif(file => includeExtensions.includes(file.extname.toLowerCase()), gulpBrotli.compress()))
        .pipe(dest(destPath))
        .on("finish", done);
};

const markGzip = (done) => {
    fs.writeFileSync(path.join(destPath, '.EXTENSION'), 'gz');
    fs.writeFileSync(path.join(destPath, '.ENCODING'), 'gzip');
    done();
};

const markBrotli = (done) => {
    fs.writeFileSync(path.join(destPath, '.EXTENSION'), 'br');
    fs.writeFileSync(path.join(destPath, '.ENCODING'), 'br');
    done();
};

task('gzip', series(clean, gzipify, markGzip));
task('brotli', series(clean, brotlify, markBrotli));