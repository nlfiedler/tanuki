//
// https://blog.sixeyed.com/docker-healthchecks-why-not-to-use-curl-or-iwr/
//
const http = require('http')

const port = process.argv.length > 2 ? process.argv[2] : '3000'
const options = {
  host: 'localhost',
  port,
  timeout: 2000
}

const request = http.request(options, (res) => {
  console.log(`STATUS: ${res.statusCode}`)
  if (res.statusCode === 200) {
    process.exit(0)
  } else {
    process.exit(1)
  }
})

request.on('error', function (err) {
  console.log('ERROR:', err)
  process.exit(1)
})

request.end()
