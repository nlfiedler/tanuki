const { exec } = require('child_process')
const del = require('del')
const gulp = require('gulp')
const gulpif = require('gulp-if')
const uglify = require('gulp-uglify')
const nodemon = require('gulp-nodemon')
const webpack = require('webpack-stream')

const production = false

function serve (cb) {
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
}

function cleanbsb (cb) {
  exec('npx bsb -clean-world', (err, stdout, stderr) => {
    console.info(stdout)
    console.error(stderr)
    cb(err)
  })
}

function cleanweb (cb) {
  return del(['public/javascripts/main.js'])
}

function makebsb (cb) {
  exec('npx bsb -make-world', (err, stdout, stderr) => {
    console.info(stdout)
    console.error(stderr)
    cb(err)
  })
}

function makeweb (cb) {
  return gulp.src('lib/js/src/Index.bs.js')
    .pipe(webpack({
      mode: production ? 'production' : 'development',
      output: {
        filename: 'main.js'
      }
    }))
    .pipe(gulpif(production, uglify()))
    .pipe(gulp.dest('public/javascripts'))
}

exports.clean = gulp.parallel(cleanbsb, cleanweb)
exports.build = gulp.series(makebsb, makeweb)

function watchServer () {
  gulp.watch('src/**/*.re', gulp.series('build'))
}
exports.default = gulp.series(exports.build, serve, watchServer)
