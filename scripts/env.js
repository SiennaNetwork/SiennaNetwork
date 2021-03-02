module.exports = (
  file = require('path').resolve(__dirname, '../.env')
) => {
  require('dotenv').config({ path: file })
  return {
    file,
    write (k,v) {
      v = `${k}=${JSON.stringify(v)}`
      require('fs').appendFileSync(
        file,
        `\n${v}`
      )
      return v
    }
  }
}
