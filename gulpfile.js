const {exec} = require('child_process')
const fs = require('fs-extra')
const gulp = require('gulp')
const gulpif = require('gulp-if')
const uglify = require('gulp-uglify')
const nodemon = require('gulp-nodemon')
const webpack = require('webpack-stream')

let production = false

gulp.task('serve', (cb) => {
  let called = false
  return nodemon({
    'script': './bin/www',
    'watch': '.',
    'ext': 'js'
  }).on('start', () => {
    if (!called) {
      called = true
      cb()
    }
  })
})

gulp.task('bsb-clean', (cb) => {
  exec('bsb -clean-world', (err, stdout, stderr) => {
    console.info(stdout)
    console.error(stderr)
    cb(err)
  })
})

gulp.task('js-clean', (cb) => {
  fs.remove('public/javascripts/main.js', err => {
    cb(err)
  })
})

gulp.task('bsb-make', (cb) => {
  exec('bsb -make-world', (err, stdout, stderr) => {
    console.info(stdout)
    console.error(stderr)
    cb(err)
  })
})

gulp.task('webpack', () => {
  return gulp.src('lib/js/src/main.bs.js')
    .pipe(webpack({
      mode: production ? 'production' : 'development',
      output: {
        filename: 'main.js'
      }
    }))
    .pipe(gulpif(production, uglify()))
    .pipe(gulp.dest('public/javascripts'))
})

gulp.task('watch-server', () => {
  gulp.watch('src/**/*.re', gulp.series('compile'))
})

gulp.task('compile', gulp.series('bsb-make', 'webpack'))
gulp.task('clean', gulp.series('bsb-clean', 'js-clean'))
gulp.task('default', gulp.series('compile', 'serve', 'watch-server'))
