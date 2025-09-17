import { src, dest, series, task } from 'gulp';
import gulpGzip from 'gulp-gzip';
import gulpBrotli from 'gulp-brotli';
import gulpif from 'gulp-if';
import del from 'delete';
import fs from 'node:fs';
import path from 'node:path';

const destPath = './dist';

const includeExtensions = ['.json', '.html', '.mjs', '.js', '.css', '.svg', '.ttf', '.woff', '.woff2', '.eot'];

const clean = (cb) => {
    del([destPath], cb);
};

const gzipify = () => {
    return src('./dist_rsbuild/**/*', {
        encoding: false
    })
        .pipe(gulpif(file => {
            return includeExtensions.includes(file.extname.toLowerCase());
        }, gulpGzip()))
        .pipe(dest(destPath))
        .on('finish', () => {
            fs.writeFileSync(path.join(destPath, '.EXTENSION'), 'gz');
            fs.writeFileSync(path.join(destPath, '.ENCODING'), 'gzip');
        });
};

const brotlify = () => {
     return src('./dist_rsbuild/**/*', {
        encoding: false
    })
        .pipe(gulpif(file => {
            return includeExtensions.includes(file.extname.toLowerCase());
        }, gulpBrotli.compress()))
        .pipe(dest(destPath))
        .on('finish', () => {
            fs.writeFileSync(path.join(destPath, '.EXTENSION'), 'br');
            fs.writeFileSync(path.join(destPath, '.ENCODING'), 'br');
        });
};

task('gzip', series(clean, gzipify));
task('brotli', series(clean, brotlify));