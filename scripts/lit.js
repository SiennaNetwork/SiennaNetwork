#!/usr/bin/env node
require('colors')
const { readFileSync } = require('fs')
const { resolve } = require('path')

const source = resolve(__dirname, 'compare.js')

const lines = []
for (let line of readFileSync(source, 'utf8').split('\n').slice(2)) {
  // extract the rightmost comment
  const columns = line.split('//')
  const comment = columns.length > 1 ? columns.pop() : ''
  const code    = columns.join('//')
  const pair    = [code, comment]
  lines.push(pair)
}

const blocks = []
let currentType = null
let currentBlock = []
for (let [code, comment] of lines) {
  let type
  if (code.trim().length > 0 && comment.trim().length > 0) type = 'mixed'
  else if (code.trim().length > 0) type = 'code'
  else type = 'text' // blank lines, significant for markdown

  if (!currentType) currentType = type
  if (currentType && currentType !== type) {
    blocks.push({type: currentType, block: currentBlock})
    currentType  = type
    currentBlock = []
  }
  currentBlock.push([code, comment])
  if (currentType === 'mixed') {
    blocks.push({type: currentType, block: currentBlock})
    currentType  = type
    currentBlock = []
  }
}

const marked = require('marked')
let output = '<!doctype html><html>'
output += '<head><style>'
output += '* { position: relative; box-sizing: border-box }'
output += 'body { font-family: sans-serif; background: #aaa; color: #333; }'
output += '.text code { font-size: 1.2em; background: white; color: #f88212; font-weight: bold; display: inline-block; padding: 0.2rem 0.5rem; margin: -0.2rem 0 }'
output += 'header, .section { max-width: 1024px; margin: 0 auto }'
output += 'header { padding-top:2em; text-align: center; background-image: linear-gradient(to bottom, transparent, #eee) }'
output += '.section.text { padding: 1em 4em }'
output += '.section.mixed { display: flex; flex-flow: row nowrap; align-items: stretch }'
output += '.section.mixed .code, .section.mixed .text { display: flex; align-items: center }'
output += '.section.mixed .code { padding: 0.5em 2em; width: 62% }'
output += '.section.mixed .text { padding: 0.5em 2em; width: 38% }'
output += '.section.mixed .text > ul { margin-left: 1em }'
output += '.section.mixed ul, .section.mixed li { padding-top: 0; padding-bottom: 0; margin: 0 }'
output += '.text { background: #eee }'
output += '.code { padding: 1em 2em; font-family: monospace; font-size: 1.2em; white-space: pre; background: #333; color: #fff }'
output += 'ul, li { margin: 0; padding: 0 }'
output += 'ul { margin: 0; padding-left: 1em }'
output += 'li > ul { margin-left: 1em; margin-top: 1em }'
output += 'li { margin-left: 1emcenter; }'
output += 'li + li { margin-top: 1em; }'
output += 'p, li { line-height: 1.5 }'
output += 'h1,h2,h3 { text-align: center }'
output += '</style></head>'
output += '<body>'
output += '<header><a href="https://hack.bg"><img src=" data:image/png;base64,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"></a></header>'
for (const {type, block} of blocks) {
  if (type === 'text') {
    const lines = []
    for (const [_, comment] of block) {
      lines.push(comment)
    }
    output += '<div class="section text">'
    output += marked(lines.join('\n'))
    output += '</div>'
  }
  if (type === 'code') {
    const lines = []
    for (const [code, _] of block) {
      lines.push(code)
    }
    output += '<div class="section code">'
    output += lines.join('\n')
    output += '</div>'
  }
  if (type === 'mixed') {
    const codeLines = [], textLines = []
    for (const [code, comment] of block) {
      codeLines.push(code)
      textLines.push(comment)
    }
    const code = codeLines.join('\n')
    const text = marked(textLines.join('\n'))
    if (code.trim().length > 0 || text.trim().length > 0) {
      output += '<div class="section mixed">'
      output += `<div class="code">${code}</div>`
      output += `<div class="text">${text}</div>`
      output += '</div>'
    }
  }
  output += '</div>'
}
output += '</body></html>'

console.log(output)
