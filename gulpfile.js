const { exec } = require('child_process')
const del = require('del')
const gulp = require('gulp')
const webpack = require('webpack-stream')

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
      mode: process.env.BUILD_ENV || 'development',
      output: {
        filename: 'main.js'
      }
    }))
    .pipe(gulp.dest('public/javascripts'))
}

exports.clean = gulp.parallel(cleanbsb, cleanweb)
exports.build = gulp.series(makebsb, makeweb)
