module.exports = (x={}) => {
  if (x.data instanceof Uint8Array) x.data = new TextDecoder('utf-8').decode(x.data)
  console.log(require('prettyjson').render(x))
}
