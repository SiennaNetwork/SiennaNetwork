const colors = require('colors/safe')

module.exports = (function sayer (prefix = '') {

  return Object.assign(say, { tag })

  function say (x = {}) {
    if (x.data instanceof Uint8Array) x.data = new TextDecoder('utf-8').decode(x.data)
    if (x instanceof Object) { // objects go on a separate line
      console.log(colors.yellow(`\n${prefix}`))
      console.log(require('prettyjson').render(x))
    } else {
      console.log(
        colors.yellow(`\n${prefix}`),
        require('prettyjson').render(x)
      )
    }
    return x
  }

  function tag (x) {
    return sayer(`${prefix}${x}`)
  }

})()

//module.exports = (
  //prefix="",
  ////color=random
//) => {
  //i[prefix] = i[prefix]||0

  //// return actual logger function:
  //return (x={}) => {
    //if (x.data instanceof Uint8Array) x.data = new TextDecoder('utf-8').decode(x.data)
    //if (x instanceof Object) { // objects go on a separate line
      //console.log(`\n${prefix} ${i[prefix]++}`)
      //console.log(require('prettyjson').render(x))
    //} else {
      //console.log(
        //`\n${prefix} ${i[prefix]++}`,
        //require('prettyjson').render(x)
      //)
    //}
    //return x
  //}
//}
