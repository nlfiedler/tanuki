const { exec } = require('child_process')
const fx = require('fs-extra')
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

gulp.task('bsb-clean', (cb) => {
  exec('npx bsb -clean-world', (err, stdout, stderr) => {
    console.info(stdout)
    console.error(stderr)
    cb(err)
  })
})

gulp.task('js-clean', (cb) => {
  fx.remove('public/javascripts/main.js', err => {
    if (err) {
      cb(err)
    } else {
      fx.remove('dist', err => {
        cb(err)
      })
    }
  })
})

gulp.task('compile', () => {
  return tsProject.src()
    .pipe(tsProject())
    .js.pipe(gulp.dest('dist'))
})

gulp.task('bsb-make', (cb) => {
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

gulp.task('compile', gulp.series('compile', 'bsb-make', 'webpack'))
gulp.task('clean', gulp.series('bsb-clean', 'js-clean'))
gulp.task('default', gulp.series('compile', 'serve', 'watch-server'))
