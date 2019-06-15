const { exec } = require('child_process')
const del = require('del')
const gulp = require('gulp')
const gulpif = require('gulp-if')
const uglify = require('gulp-uglify')
const nodemon = require('gulp-nodemon')
const webpack = require('webpack-stream')
const ts = require('gulp-typescript')
const tsProject = ts.createProject('tsconfig.json')

let production = false

gulp.task('serve', (cb) => {
  let called = false
  return nodemon({
    'script': './dist/server.js',
    'env': {
      'NODE_PATH': '.'
    },
    'watch': './dist',
    'ext': 'js'
  }).on('start', () => {
    if (!called) {
      called = true
      cb()
    }
  })
})

gulp.task('clean:bsb', (cb) => {
  exec('npx bsb -clean-world', (err, stdout, stderr) => {
    console.info(stdout)
    console.error(stderr)
    cb(err)
  })
})

gulp.task('clean:js', (cb) => {
  return del(['public/javascripts/main.js', 'dist'])
})

gulp.task('compile', () => {
  return tsProject.src()
    .pipe(tsProject())
    .js.pipe(gulp.dest('dist'))
})

gulp.task('make:bsb', (cb) => {
  exec('npx bsb -make-world', (err, stdout, stderr) => {
    console.info(stdout)
    console.error(stderr)
    cb(err)
  })
})

gulp.task('webpack', () => {
  return gulp.src('lib/js/web/main.bs.js')
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
  gulp.watch('web/**/*.re', gulp.series('compile'))
})

gulp.task('build', gulp.series('compile', 'make:bsb', 'webpack'))
gulp.task('clean', gulp.series('clean:bsb', 'clean:js'))
gulp.task('default', gulp.series('build', 'serve', 'watch-server'))
