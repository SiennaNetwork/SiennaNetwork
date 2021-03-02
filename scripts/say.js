const i = {}
module.exports = (
  prefix="",
  //color=random
) => {
  i[prefix] = i[prefix]||0
  return (x={}) => {
    if (x.data instanceof Uint8Array) x.data = new TextDecoder('utf-8').decode(x.data)
    console.log(prefix, i[prefix]++, require('prettyjson').render(x))
    return x
  }
}
