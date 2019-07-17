const { exec } = require('child_process')
const del = require('del')
const gulp = require('gulp')
const gulpif = require('gulp-if')
const uglify = require('gulp-uglify')
const nodemon = require('gulp-nodemon')
const webpack = require('webpack-stream')

const production = false

gulp.task('serve', (cb) => {
  let called = false
  return nodemon({
    script: './src/server.js',
    env: {
      NODE_PATH: 'src'
    },
    watch: './src',
    ext: 'js'
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

gulp.task('clean:web', (cb) => {
  return del(['public/javascripts/main.js'])
})

gulp.task('make:bsb', (cb) => {
  exec('npx bsb -make-world', (err, stdout, stderr) => {
    console.info(stdout)
    console.error(stderr)
    cb(err)
  })
})

gulp.task('make:web', () => {
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
  gulp.watch('src/**/*.re', gulp.series('make:bsb'))
})

gulp.task('build', gulp.series('make:bsb', 'make:web'))
gulp.task('clean', gulp.series('clean:bsb', 'clean:web'))
gulp.task('default', gulp.series('build', 'serve', 'watch-server'))
