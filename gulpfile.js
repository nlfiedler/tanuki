const gulp = require('gulp')
const gulpif = require('gulp-if')
const uglify = require('gulp-uglify')
const nodemon = require('gulp-nodemon')
const elm = require('gulp-elm')

let production = false

gulp.task('serve', function (cb) {
  let called = false
  return nodemon({
    'script': './bin/www',
    'watch': '.',
    'ext': 'js'
  }).on('start', function () {
    if (!called) {
      called = true
      cb()
    }
  })
})

gulp.task('elm-init', elm.init)

gulp.task('elm-compile', ['elm-init'], function () {
  return gulp.src('elm-src/Main.elm')
    .pipe(elm({'warn': true}))
    .pipe(gulpif(production, uglify()))
    .pipe(gulp.dest('public/javascripts'))
})

gulp.task('watch-server', ['serve'], function () {
  gulp.watch('elm-src/**/*.elm', ['elm-compile'])
})

gulp.task('default', ['elm-compile', 'watch-server'])

//
// For more ideas and weird examples...
//
// https://github.com/simonh1000/elm-fullstack-starter
//
