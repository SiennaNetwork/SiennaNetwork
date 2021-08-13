/******/ (() => { // webpackBootstrap
/******/ 	"use strict";
/******/ 	var __webpack_modules__ = ({

/***/ "./target/web/rewards.js":
/*!*******************************!*\
  !*** ./target/web/rewards.js ***!
  \*******************************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

__webpack_require__.r(__webpack_exports__);
/* harmony export */ __webpack_require__.d(__webpack_exports__, {
/* harmony export */   "Contract": () => (/* binding */ Contract),
/* harmony export */   "default": () => (__WEBPACK_DEFAULT_EXPORT__)
/* harmony export */ });

let wasm;

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachegetUint8Memory0 = null;
function getUint8Memory0() {
    if (cachegetUint8Memory0 === null || cachegetUint8Memory0.buffer !== wasm.memory.buffer) {
        cachegetUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachegetUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

const heap = new Array(32).fill(undefined);

heap.push(undefined, null, true, false);

let heap_next = heap.length;

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    if (typeof(heap_next) !== 'number') throw new Error('corrupt heap');

    heap[idx] = obj;
    return idx;
}

function getObject(idx) { return heap[idx]; }

function dropObject(idx) {
    if (idx < 36) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

function _assertNum(n) {
    if (typeof(n) !== 'number') throw new Error('expected a number argument');
}

let WASM_VECTOR_LEN = 0;

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1);
    getUint8Memory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

const u32CvtShim = new Uint32Array(2);

const uint64CvtShim = new BigUint64Array(u32CvtShim.buffer);

let cachegetInt32Memory0 = null;
function getInt32Memory0() {
    if (cachegetInt32Memory0 === null || cachegetInt32Memory0.buffer !== wasm.memory.buffer) {
        cachegetInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachegetInt32Memory0;
}

function getArrayU8FromWasm0(ptr, len) {
    return getUint8Memory0().subarray(ptr / 1, ptr / 1 + len);
}
/**
*/
class Contract {

    static __wrap(ptr) {
        const obj = Object.create(Contract.prototype);
        obj.ptr = ptr;

        return obj;
    }

    toJSON() {
        return {
            get_block: this.get_block,
        };
    }

    toString() {
        return JSON.stringify(this);
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_contract_free(ptr);
    }
    /**
    */
    constructor() {
        var ret = wasm.contract_new();
        return Contract.__wrap(ret);
    }
    /**
    * @param {Uint8Array} sender
    */
    set sender(sender) {
        if (this.ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.ptr);
        var ptr0 = passArray8ToWasm0(sender, wasm.__wbindgen_malloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.contract_set_sender(this.ptr, ptr0, len0);
    }
    /**
    * @param {BigInt} height
    */
    set block(height) {
        if (this.ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.ptr);
        uint64CvtShim[0] = height;
        const low0 = u32CvtShim[0];
        const high0 = u32CvtShim[1];
        wasm.contract_set_block(this.ptr, low0, high0);
    }
    /**
    * @returns {BigInt}
    */
    get get_block() {
        try {
            if (this.ptr == 0) throw new Error('Attempt to use a moved value');
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            _assertNum(this.ptr);
            wasm.contract_get_block(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            u32CvtShim[0] = r0;
            u32CvtShim[1] = r1;
            const n0 = uint64CvtShim[0];
            return n0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * @param {Uint8Array} response
    */
    set next_query_response(response) {
        if (this.ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.ptr);
        var ptr0 = passArray8ToWasm0(response, wasm.__wbindgen_malloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.contract_set_next_query_response(this.ptr, ptr0, len0);
    }
    /**
    * @param {Uint8Array} msg
    * @returns {Uint8Array}
    */
    init(msg) {
        try {
            if (this.ptr == 0) throw new Error('Attempt to use a moved value');
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            _assertNum(this.ptr);
            var ptr0 = passArray8ToWasm0(msg, wasm.__wbindgen_malloc);
            var len0 = WASM_VECTOR_LEN;
            wasm.contract_init(retptr, this.ptr, ptr0, len0);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v1 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 1);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * @param {Uint8Array} msg
    * @returns {Uint8Array}
    */
    handle(msg) {
        try {
            if (this.ptr == 0) throw new Error('Attempt to use a moved value');
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            _assertNum(this.ptr);
            var ptr0 = passArray8ToWasm0(msg, wasm.__wbindgen_malloc);
            var len0 = WASM_VECTOR_LEN;
            wasm.contract_handle(retptr, this.ptr, ptr0, len0);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v1 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 1);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * @param {Uint8Array} msg
    * @returns {Uint8Array}
    */
    query(msg) {
        try {
            if (this.ptr == 0) throw new Error('Attempt to use a moved value');
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            _assertNum(this.ptr);
            var ptr0 = passArray8ToWasm0(msg, wasm.__wbindgen_malloc);
            var len0 = WASM_VECTOR_LEN;
            wasm.contract_query(retptr, this.ptr, ptr0, len0);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v1 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 1);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}

async function load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

async function init(input) {
    if (typeof input === 'undefined') {
        input = new URL(/* asset import */ __webpack_require__(/*! rewards_bg.wasm */ "./target/web/rewards_bg.wasm"), __webpack_require__.b);
    }
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
        var ret = getStringFromWasm0(arg0, arg1);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbindgen_rethrow = function(arg0) {
        throw takeObject(arg0);
    };

    if (typeof input === 'string' || (typeof Request === 'function' && input instanceof Request) || (typeof URL === 'function' && input instanceof URL)) {
        input = fetch(input);
    }



    const { instance, module } = await load(await input, imports);

    wasm = instance.exports;
    init.__wbindgen_wasm_module = module;

    return wasm;
}

/* harmony default export */ const __WEBPACK_DEFAULT_EXPORT__ = (init);



/***/ }),

/***/ "./target/web/rewards_bg.wasm":
/*!************************************!*\
  !*** ./target/web/rewards_bg.wasm ***!
  \************************************/
/***/ ((module, __unused_webpack_exports, __webpack_require__) => {

module.exports = __webpack_require__.p + "e7c93f95f36554ca08e1.wasm";

/***/ }),

/***/ "../../node_modules/css-loader/dist/cjs.js!./dashboard/style.css":
/*!***********************************************************************!*\
  !*** ../../node_modules/css-loader/dist/cjs.js!./dashboard/style.css ***!
  \***********************************************************************/
/***/ ((module, __webpack_exports__, __webpack_require__) => {

__webpack_require__.r(__webpack_exports__);
/* harmony export */ __webpack_require__.d(__webpack_exports__, {
/* harmony export */   "default": () => (__WEBPACK_DEFAULT_EXPORT__)
/* harmony export */ });
/* harmony import */ var _node_modules_css_loader_dist_runtime_cssWithMappingToString_js__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ../../../node_modules/css-loader/dist/runtime/cssWithMappingToString.js */ "../../node_modules/css-loader/dist/runtime/cssWithMappingToString.js");
/* harmony import */ var _node_modules_css_loader_dist_runtime_cssWithMappingToString_js__WEBPACK_IMPORTED_MODULE_0___default = /*#__PURE__*/__webpack_require__.n(_node_modules_css_loader_dist_runtime_cssWithMappingToString_js__WEBPACK_IMPORTED_MODULE_0__);
/* harmony import */ var _node_modules_css_loader_dist_runtime_api_js__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ../../../node_modules/css-loader/dist/runtime/api.js */ "../../node_modules/css-loader/dist/runtime/api.js");
/* harmony import */ var _node_modules_css_loader_dist_runtime_api_js__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(_node_modules_css_loader_dist_runtime_api_js__WEBPACK_IMPORTED_MODULE_1__);
// Imports


var ___CSS_LOADER_EXPORT___ = _node_modules_css_loader_dist_runtime_api_js__WEBPACK_IMPORTED_MODULE_1___default()((_node_modules_css_loader_dist_runtime_cssWithMappingToString_js__WEBPACK_IMPORTED_MODULE_0___default()));
// Module
___CSS_LOADER_EXPORT___.push([module.id, "* {\n  box-sizing: border-box;\n  margin: 0;\n  padding: 0;\n  font-size: 1rem;\n  list-style: none;\n}\n\nhtml, body {\n  height:  100%;\n  margin:  0;\n  padding: 0;\n  font-family: sans-serif;\n}\n\nbody {\n  background: #282828;\n  color:      #ebdbb2;\n  display:    grid;\n  grid-template-columns: 20% 20% 20% 20% 20%;\n  grid-template-rows: 50% 50%\n}\n\n@media (orientation: landscape) {}\n@media (orientation: portrait) {}\n\nh1 {\n  font-weight: bold;\n  margin: 0;\n}\nh2 {\n  font-weight: normal;\n  text-align: right;\n  margin-bottom: 1rem;\n}\n\n.pie, .history {\n  /*flex-grow: 1;*/\n  /*flex-shrink: 1;*/\n  /*flex-basis: 100%;*/\n}\n\n.pie {\n  text-align: left;\n  grid-column-start: 1;\n  grid-column-end:   3;\n}\n.pie.stacked {\n  grid-column-start: 3;\n  grid-column-end:   5;\n}\n  .pie h1 {\n    margin: 1em 1em 0 1em;\n  }\n  .pie canvas {\n  }\n\ntable {\n  grid-row-start: 2;\n  grid-column-start: 1;\n  grid-column-end: 5;\n  background: #32302f;\n  border-collapse: collapse;\n}\n\ntd {\n  padding: 0.25rem;\n  text-align: center;\n}\ntd.claimable:hover {\n  color: white !important;\n  cursor: pointer;\n  text-decoration: underline;\n}\nthead {\n  background: #504945;\n}\nth {\n  padding: 0.5rem;\n}\n\n.history {\n  grid-row-start: 1;\n  grid-row-end: 3;\n  grid-column-start: 5;\n  padding: 1em;\n  background: #3c3836;\n  overflow: auto;\n}\n\nh1, h2, ol {\n  padding: 0 0.5em;\n}\n\n/*.sparkline {*/\n  /*position: absolute;*/\n  /*left: 0;*/\n  /*right: 0;*/\n  /*bottom: 0;*/\n  /*height: 20vh;*/\n/*}*/\n\ncenter {\n  grid-column-start: 1;\n  grid-column-end: 6;\n  grid-row-start: 1;\n  grid-row-end: 3;\n  display: flex;\n  align-items: center;\n  justify-content: center;\n  font-size: 3rem;\n}\n\n.field {\n  padding-bottom: 1rem;\n  border-bottom: 1px solid rgba(0,0,0,0.5);\n  margin-bottom: 1rem;\n}\n.field label {\n  font-weight: bold;\n}\n.field > div {\n  text-align: right\n}\n\ntd.locked {\n  display: flex;\n  justify-content: space-between;\n  background: rgba(0,0,0,0.2)\n}\n.locked button {\n  padding: 0 1rem;\n}\n", "",{"version":3,"sources":["webpack://./dashboard/style.css"],"names":[],"mappings":"AAAA;EACE,sBAAsB;EACtB,SAAS;EACT,UAAU;EACV,eAAe;EACf,gBAAgB;AAClB;;AAEA;EACE,aAAa;EACb,UAAU;EACV,UAAU;EACV,uBAAuB;AACzB;;AAEA;EACE,mBAAmB;EACnB,mBAAmB;EACnB,gBAAgB;EAChB,0CAA0C;EAC1C;AACF;;AAEA,iCAAiC;AACjC,gCAAgC;;AAEhC;EACE,iBAAiB;EACjB,SAAS;AACX;AACA;EACE,mBAAmB;EACnB,iBAAiB;EACjB,mBAAmB;AACrB;;AAEA;EACE,gBAAgB;EAChB,kBAAkB;EAClB,oBAAoB;AACtB;;AAEA;EACE,gBAAgB;EAChB,oBAAoB;EACpB,oBAAoB;AACtB;AACA;EACE,oBAAoB;EACpB,oBAAoB;AACtB;EACE;IACE,qBAAqB;EACvB;EACA;EACA;;AAEF;EACE,iBAAiB;EACjB,oBAAoB;EACpB,kBAAkB;EAClB,mBAAmB;EACnB,yBAAyB;AAC3B;;AAEA;EACE,gBAAgB;EAChB,kBAAkB;AACpB;AACA;EACE,uBAAuB;EACvB,eAAe;EACf,0BAA0B;AAC5B;AACA;EACE,mBAAmB;AACrB;AACA;EACE,eAAe;AACjB;;AAEA;EACE,iBAAiB;EACjB,eAAe;EACf,oBAAoB;EACpB,YAAY;EACZ,mBAAmB;EACnB,cAAc;AAChB;;AAEA;EACE,gBAAgB;AAClB;;AAEA,eAAe;EACb,sBAAsB;EACtB,WAAW;EACX,YAAY;EACZ,aAAa;EACb,gBAAgB;AAClB,IAAI;;AAEJ;EACE,oBAAoB;EACpB,kBAAkB;EAClB,iBAAiB;EACjB,eAAe;EACf,aAAa;EACb,mBAAmB;EACnB,uBAAuB;EACvB,eAAe;AACjB;;AAEA;EACE,oBAAoB;EACpB,wCAAwC;EACxC,mBAAmB;AACrB;AACA;EACE,iBAAiB;AACnB;AACA;EACE;AACF;;AAEA;EACE,aAAa;EACb,8BAA8B;EAC9B;AACF;AACA;EACE,eAAe;AACjB","sourcesContent":["* {\n  box-sizing: border-box;\n  margin: 0;\n  padding: 0;\n  font-size: 1rem;\n  list-style: none;\n}\n\nhtml, body {\n  height:  100%;\n  margin:  0;\n  padding: 0;\n  font-family: sans-serif;\n}\n\nbody {\n  background: #282828;\n  color:      #ebdbb2;\n  display:    grid;\n  grid-template-columns: 20% 20% 20% 20% 20%;\n  grid-template-rows: 50% 50%\n}\n\n@media (orientation: landscape) {}\n@media (orientation: portrait) {}\n\nh1 {\n  font-weight: bold;\n  margin: 0;\n}\nh2 {\n  font-weight: normal;\n  text-align: right;\n  margin-bottom: 1rem;\n}\n\n.pie, .history {\n  /*flex-grow: 1;*/\n  /*flex-shrink: 1;*/\n  /*flex-basis: 100%;*/\n}\n\n.pie {\n  text-align: left;\n  grid-column-start: 1;\n  grid-column-end:   3;\n}\n.pie.stacked {\n  grid-column-start: 3;\n  grid-column-end:   5;\n}\n  .pie h1 {\n    margin: 1em 1em 0 1em;\n  }\n  .pie canvas {\n  }\n\ntable {\n  grid-row-start: 2;\n  grid-column-start: 1;\n  grid-column-end: 5;\n  background: #32302f;\n  border-collapse: collapse;\n}\n\ntd {\n  padding: 0.25rem;\n  text-align: center;\n}\ntd.claimable:hover {\n  color: white !important;\n  cursor: pointer;\n  text-decoration: underline;\n}\nthead {\n  background: #504945;\n}\nth {\n  padding: 0.5rem;\n}\n\n.history {\n  grid-row-start: 1;\n  grid-row-end: 3;\n  grid-column-start: 5;\n  padding: 1em;\n  background: #3c3836;\n  overflow: auto;\n}\n\nh1, h2, ol {\n  padding: 0 0.5em;\n}\n\n/*.sparkline {*/\n  /*position: absolute;*/\n  /*left: 0;*/\n  /*right: 0;*/\n  /*bottom: 0;*/\n  /*height: 20vh;*/\n/*}*/\n\ncenter {\n  grid-column-start: 1;\n  grid-column-end: 6;\n  grid-row-start: 1;\n  grid-row-end: 3;\n  display: flex;\n  align-items: center;\n  justify-content: center;\n  font-size: 3rem;\n}\n\n.field {\n  padding-bottom: 1rem;\n  border-bottom: 1px solid rgba(0,0,0,0.5);\n  margin-bottom: 1rem;\n}\n.field label {\n  font-weight: bold;\n}\n.field > div {\n  text-align: right\n}\n\ntd.locked {\n  display: flex;\n  justify-content: space-between;\n  background: rgba(0,0,0,0.2)\n}\n.locked button {\n  padding: 0 1rem;\n}\n"],"sourceRoot":""}]);
// Exports
/* harmony default export */ const __WEBPACK_DEFAULT_EXPORT__ = (___CSS_LOADER_EXPORT___);


/***/ }),

/***/ "../../node_modules/css-loader/dist/runtime/api.js":
/*!*********************************************************!*\
  !*** ../../node_modules/css-loader/dist/runtime/api.js ***!
  \*********************************************************/
/***/ ((module) => {



/*
  MIT License http://www.opensource.org/licenses/mit-license.php
  Author Tobias Koppers @sokra
*/
// css base code, injected by the css-loader
// eslint-disable-next-line func-names
module.exports = function (cssWithMappingToString) {
  var list = []; // return the list of modules as css string

  list.toString = function toString() {
    return this.map(function (item) {
      var content = cssWithMappingToString(item);

      if (item[2]) {
        return "@media ".concat(item[2], " {").concat(content, "}");
      }

      return content;
    }).join("");
  }; // import a list of modules into the list
  // eslint-disable-next-line func-names


  list.i = function (modules, mediaQuery, dedupe) {
    if (typeof modules === "string") {
      // eslint-disable-next-line no-param-reassign
      modules = [[null, modules, ""]];
    }

    var alreadyImportedModules = {};

    if (dedupe) {
      for (var i = 0; i < this.length; i++) {
        // eslint-disable-next-line prefer-destructuring
        var id = this[i][0];

        if (id != null) {
          alreadyImportedModules[id] = true;
        }
      }
    }

    for (var _i = 0; _i < modules.length; _i++) {
      var item = [].concat(modules[_i]);

      if (dedupe && alreadyImportedModules[item[0]]) {
        // eslint-disable-next-line no-continue
        continue;
      }

      if (mediaQuery) {
        if (!item[2]) {
          item[2] = mediaQuery;
        } else {
          item[2] = "".concat(mediaQuery, " and ").concat(item[2]);
        }
      }

      list.push(item);
    }
  };

  return list;
};

/***/ }),

/***/ "../../node_modules/css-loader/dist/runtime/cssWithMappingToString.js":
/*!****************************************************************************!*\
  !*** ../../node_modules/css-loader/dist/runtime/cssWithMappingToString.js ***!
  \****************************************************************************/
/***/ ((module) => {



function _slicedToArray(arr, i) { return _arrayWithHoles(arr) || _iterableToArrayLimit(arr, i) || _unsupportedIterableToArray(arr, i) || _nonIterableRest(); }

function _nonIterableRest() { throw new TypeError("Invalid attempt to destructure non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method."); }

function _unsupportedIterableToArray(o, minLen) { if (!o) return; if (typeof o === "string") return _arrayLikeToArray(o, minLen); var n = Object.prototype.toString.call(o).slice(8, -1); if (n === "Object" && o.constructor) n = o.constructor.name; if (n === "Map" || n === "Set") return Array.from(o); if (n === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)) return _arrayLikeToArray(o, minLen); }

function _arrayLikeToArray(arr, len) { if (len == null || len > arr.length) len = arr.length; for (var i = 0, arr2 = new Array(len); i < len; i++) { arr2[i] = arr[i]; } return arr2; }

function _iterableToArrayLimit(arr, i) { var _i = arr && (typeof Symbol !== "undefined" && arr[Symbol.iterator] || arr["@@iterator"]); if (_i == null) return; var _arr = []; var _n = true; var _d = false; var _s, _e; try { for (_i = _i.call(arr); !(_n = (_s = _i.next()).done); _n = true) { _arr.push(_s.value); if (i && _arr.length === i) break; } } catch (err) { _d = true; _e = err; } finally { try { if (!_n && _i["return"] != null) _i["return"](); } finally { if (_d) throw _e; } } return _arr; }

function _arrayWithHoles(arr) { if (Array.isArray(arr)) return arr; }

module.exports = function cssWithMappingToString(item) {
  var _item = _slicedToArray(item, 4),
      content = _item[1],
      cssMapping = _item[3];

  if (!cssMapping) {
    return content;
  }

  if (typeof btoa === "function") {
    // eslint-disable-next-line no-undef
    var base64 = btoa(unescape(encodeURIComponent(JSON.stringify(cssMapping))));
    var data = "sourceMappingURL=data:application/json;charset=utf-8;base64,".concat(base64);
    var sourceMapping = "/*# ".concat(data, " */");
    var sourceURLs = cssMapping.sources.map(function (source) {
      return "/*# sourceURL=".concat(cssMapping.sourceRoot || "").concat(source, " */");
    });
    return [content].concat(sourceURLs).concat([sourceMapping]).join("\n");
  }

  return [content].join("\n");
};

/***/ }),

/***/ "./dashboard/style.css":
/*!*****************************!*\
  !*** ./dashboard/style.css ***!
  \*****************************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

__webpack_require__.r(__webpack_exports__);
/* harmony export */ __webpack_require__.d(__webpack_exports__, {
/* harmony export */   "default": () => (__WEBPACK_DEFAULT_EXPORT__)
/* harmony export */ });
/* harmony import */ var _node_modules_style_loader_dist_runtime_injectStylesIntoStyleTag_js__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! !../../../node_modules/style-loader/dist/runtime/injectStylesIntoStyleTag.js */ "../../node_modules/style-loader/dist/runtime/injectStylesIntoStyleTag.js");
/* harmony import */ var _node_modules_style_loader_dist_runtime_injectStylesIntoStyleTag_js__WEBPACK_IMPORTED_MODULE_0___default = /*#__PURE__*/__webpack_require__.n(_node_modules_style_loader_dist_runtime_injectStylesIntoStyleTag_js__WEBPACK_IMPORTED_MODULE_0__);
/* harmony import */ var _node_modules_style_loader_dist_runtime_styleDomAPI_js__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! !../../../node_modules/style-loader/dist/runtime/styleDomAPI.js */ "../../node_modules/style-loader/dist/runtime/styleDomAPI.js");
/* harmony import */ var _node_modules_style_loader_dist_runtime_styleDomAPI_js__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(_node_modules_style_loader_dist_runtime_styleDomAPI_js__WEBPACK_IMPORTED_MODULE_1__);
/* harmony import */ var _node_modules_style_loader_dist_runtime_insertBySelector_js__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! !../../../node_modules/style-loader/dist/runtime/insertBySelector.js */ "../../node_modules/style-loader/dist/runtime/insertBySelector.js");
/* harmony import */ var _node_modules_style_loader_dist_runtime_insertBySelector_js__WEBPACK_IMPORTED_MODULE_2___default = /*#__PURE__*/__webpack_require__.n(_node_modules_style_loader_dist_runtime_insertBySelector_js__WEBPACK_IMPORTED_MODULE_2__);
/* harmony import */ var _node_modules_style_loader_dist_runtime_setAttributesWithoutAttributes_js__WEBPACK_IMPORTED_MODULE_3__ = __webpack_require__(/*! !../../../node_modules/style-loader/dist/runtime/setAttributesWithoutAttributes.js */ "../../node_modules/style-loader/dist/runtime/setAttributesWithoutAttributes.js");
/* harmony import */ var _node_modules_style_loader_dist_runtime_setAttributesWithoutAttributes_js__WEBPACK_IMPORTED_MODULE_3___default = /*#__PURE__*/__webpack_require__.n(_node_modules_style_loader_dist_runtime_setAttributesWithoutAttributes_js__WEBPACK_IMPORTED_MODULE_3__);
/* harmony import */ var _node_modules_style_loader_dist_runtime_insertStyleElement_js__WEBPACK_IMPORTED_MODULE_4__ = __webpack_require__(/*! !../../../node_modules/style-loader/dist/runtime/insertStyleElement.js */ "../../node_modules/style-loader/dist/runtime/insertStyleElement.js");
/* harmony import */ var _node_modules_style_loader_dist_runtime_insertStyleElement_js__WEBPACK_IMPORTED_MODULE_4___default = /*#__PURE__*/__webpack_require__.n(_node_modules_style_loader_dist_runtime_insertStyleElement_js__WEBPACK_IMPORTED_MODULE_4__);
/* harmony import */ var _node_modules_style_loader_dist_runtime_styleTagTransform_js__WEBPACK_IMPORTED_MODULE_5__ = __webpack_require__(/*! !../../../node_modules/style-loader/dist/runtime/styleTagTransform.js */ "../../node_modules/style-loader/dist/runtime/styleTagTransform.js");
/* harmony import */ var _node_modules_style_loader_dist_runtime_styleTagTransform_js__WEBPACK_IMPORTED_MODULE_5___default = /*#__PURE__*/__webpack_require__.n(_node_modules_style_loader_dist_runtime_styleTagTransform_js__WEBPACK_IMPORTED_MODULE_5__);
/* harmony import */ var _node_modules_css_loader_dist_cjs_js_style_css__WEBPACK_IMPORTED_MODULE_6__ = __webpack_require__(/*! !!../../../node_modules/css-loader/dist/cjs.js!./style.css */ "../../node_modules/css-loader/dist/cjs.js!./dashboard/style.css");

      
      
      
      
      
      
      
      
      

var options = {};

options.styleTagTransform = (_node_modules_style_loader_dist_runtime_styleTagTransform_js__WEBPACK_IMPORTED_MODULE_5___default());
options.setAttributes = (_node_modules_style_loader_dist_runtime_setAttributesWithoutAttributes_js__WEBPACK_IMPORTED_MODULE_3___default());

      options.insert = _node_modules_style_loader_dist_runtime_insertBySelector_js__WEBPACK_IMPORTED_MODULE_2___default().bind(null, "head");
    
options.domAPI = (_node_modules_style_loader_dist_runtime_styleDomAPI_js__WEBPACK_IMPORTED_MODULE_1___default());
options.insertStyleElement = (_node_modules_style_loader_dist_runtime_insertStyleElement_js__WEBPACK_IMPORTED_MODULE_4___default());

var update = _node_modules_style_loader_dist_runtime_injectStylesIntoStyleTag_js__WEBPACK_IMPORTED_MODULE_0___default()(_node_modules_css_loader_dist_cjs_js_style_css__WEBPACK_IMPORTED_MODULE_6__.default, options);




       /* harmony default export */ const __WEBPACK_DEFAULT_EXPORT__ = (_node_modules_css_loader_dist_cjs_js_style_css__WEBPACK_IMPORTED_MODULE_6__.default && _node_modules_css_loader_dist_cjs_js_style_css__WEBPACK_IMPORTED_MODULE_6__.default.locals ? _node_modules_css_loader_dist_cjs_js_style_css__WEBPACK_IMPORTED_MODULE_6__.default.locals : undefined);


/***/ }),

/***/ "../../node_modules/style-loader/dist/runtime/injectStylesIntoStyleTag.js":
/*!********************************************************************************!*\
  !*** ../../node_modules/style-loader/dist/runtime/injectStylesIntoStyleTag.js ***!
  \********************************************************************************/
/***/ ((module) => {



var stylesInDom = [];

function getIndexByIdentifier(identifier) {
  var result = -1;

  for (var i = 0; i < stylesInDom.length; i++) {
    if (stylesInDom[i].identifier === identifier) {
      result = i;
      break;
    }
  }

  return result;
}

function modulesToDom(list, options) {
  var idCountMap = {};
  var identifiers = [];

  for (var i = 0; i < list.length; i++) {
    var item = list[i];
    var id = options.base ? item[0] + options.base : item[0];
    var count = idCountMap[id] || 0;
    var identifier = "".concat(id, " ").concat(count);
    idCountMap[id] = count + 1;
    var index = getIndexByIdentifier(identifier);
    var obj = {
      css: item[1],
      media: item[2],
      sourceMap: item[3]
    };

    if (index !== -1) {
      stylesInDom[index].references++;
      stylesInDom[index].updater(obj);
    } else {
      stylesInDom.push({
        identifier: identifier,
        updater: addStyle(obj, options),
        references: 1
      });
    }

    identifiers.push(identifier);
  }

  return identifiers;
}

function addStyle(obj, options) {
  var api = options.domAPI(options);
  api.update(obj);
  return function updateStyle(newObj) {
    if (newObj) {
      if (newObj.css === obj.css && newObj.media === obj.media && newObj.sourceMap === obj.sourceMap) {
        return;
      }

      api.update(obj = newObj);
    } else {
      api.remove();
    }
  };
}

module.exports = function (list, options) {
  options = options || {};
  list = list || [];
  var lastIdentifiers = modulesToDom(list, options);
  return function update(newList) {
    newList = newList || [];

    for (var i = 0; i < lastIdentifiers.length; i++) {
      var identifier = lastIdentifiers[i];
      var index = getIndexByIdentifier(identifier);
      stylesInDom[index].references--;
    }

    var newLastIdentifiers = modulesToDom(newList, options);

    for (var _i = 0; _i < lastIdentifiers.length; _i++) {
      var _identifier = lastIdentifiers[_i];

      var _index = getIndexByIdentifier(_identifier);

      if (stylesInDom[_index].references === 0) {
        stylesInDom[_index].updater();

        stylesInDom.splice(_index, 1);
      }
    }

    lastIdentifiers = newLastIdentifiers;
  };
};

/***/ }),

/***/ "../../node_modules/style-loader/dist/runtime/insertBySelector.js":
/*!************************************************************************!*\
  !*** ../../node_modules/style-loader/dist/runtime/insertBySelector.js ***!
  \************************************************************************/
/***/ ((module) => {



var memo = {};
/* istanbul ignore next  */

function getTarget(target) {
  if (typeof memo[target] === "undefined") {
    var styleTarget = document.querySelector(target); // Special case to return head of iframe instead of iframe itself

    if (window.HTMLIFrameElement && styleTarget instanceof window.HTMLIFrameElement) {
      try {
        // This will throw an exception if access to iframe is blocked
        // due to cross-origin restrictions
        styleTarget = styleTarget.contentDocument.head;
      } catch (e) {
        // istanbul ignore next
        styleTarget = null;
      }
    }

    memo[target] = styleTarget;
  }

  return memo[target];
}
/* istanbul ignore next  */


function insertBySelector(insert, style) {
  var target = getTarget(insert);

  if (!target) {
    throw new Error("Couldn't find a style target. This probably means that the value for the 'insert' parameter is invalid.");
  }

  target.appendChild(style);
}

module.exports = insertBySelector;

/***/ }),

/***/ "../../node_modules/style-loader/dist/runtime/insertStyleElement.js":
/*!**************************************************************************!*\
  !*** ../../node_modules/style-loader/dist/runtime/insertStyleElement.js ***!
  \**************************************************************************/
/***/ ((module) => {



/* istanbul ignore next  */
function insertStyleElement(options) {
  var style = document.createElement("style");
  options.setAttributes(style, options.attributes);
  options.insert(style);
  return style;
}

module.exports = insertStyleElement;

/***/ }),

/***/ "../../node_modules/style-loader/dist/runtime/setAttributesWithoutAttributes.js":
/*!**************************************************************************************!*\
  !*** ../../node_modules/style-loader/dist/runtime/setAttributesWithoutAttributes.js ***!
  \**************************************************************************************/
/***/ ((module, __unused_webpack_exports, __webpack_require__) => {



/* istanbul ignore next  */
function setAttributesWithoutAttributes(style) {
  var nonce =  true ? __webpack_require__.nc : 0;

  if (nonce) {
    style.setAttribute("nonce", nonce);
  }
}

module.exports = setAttributesWithoutAttributes;

/***/ }),

/***/ "../../node_modules/style-loader/dist/runtime/styleDomAPI.js":
/*!*******************************************************************!*\
  !*** ../../node_modules/style-loader/dist/runtime/styleDomAPI.js ***!
  \*******************************************************************/
/***/ ((module) => {



/* istanbul ignore next  */
function apply(style, options, obj) {
  var css = obj.css;
  var media = obj.media;
  var sourceMap = obj.sourceMap;

  if (media) {
    style.setAttribute("media", media);
  } else {
    style.removeAttribute("media");
  }

  if (sourceMap && typeof btoa !== "undefined") {
    css += "\n/*# sourceMappingURL=data:application/json;base64,".concat(btoa(unescape(encodeURIComponent(JSON.stringify(sourceMap)))), " */");
  } // For old IE

  /* istanbul ignore if  */


  options.styleTagTransform(css, style);
}

function removeStyleElement(style) {
  // istanbul ignore if
  if (style.parentNode === null) {
    return false;
  }

  style.parentNode.removeChild(style);
}
/* istanbul ignore next  */


function domAPI(options) {
  var style = options.insertStyleElement(options);
  return {
    update: function update(obj) {
      apply(style, options, obj);
    },
    remove: function remove() {
      removeStyleElement(style);
    }
  };
}

module.exports = domAPI;

/***/ }),

/***/ "../../node_modules/style-loader/dist/runtime/styleTagTransform.js":
/*!*************************************************************************!*\
  !*** ../../node_modules/style-loader/dist/runtime/styleTagTransform.js ***!
  \*************************************************************************/
/***/ ((module) => {



/* istanbul ignore next  */
function styleTagTransform(css, style) {
  if (style.styleSheet) {
    style.styleSheet.cssText = css;
  } else {
    while (style.firstChild) {
      style.removeChild(style.firstChild);
    }

    style.appendChild(document.createTextNode(css));
  }
}

module.exports = styleTagTransform;

/***/ }),

/***/ "./dashboard/contract_base.ts":
/*!************************************!*\
  !*** ./dashboard/contract_base.ts ***!
  \************************************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

__webpack_require__.r(__webpack_exports__);
/* harmony export */ __webpack_require__.d(__webpack_exports__, {
/* harmony export */   "TIME_SCALE": () => (/* binding */ TIME_SCALE),
/* harmony export */   "FUND_PORTIONS": () => (/* binding */ FUND_PORTIONS),
/* harmony export */   "DIGITS": () => (/* binding */ DIGITS),
/* harmony export */   "DIGITS_INV": () => (/* binding */ DIGITS_INV),
/* harmony export */   "FUND_PORTION": () => (/* binding */ FUND_PORTION),
/* harmony export */   "FUND_INTERVAL": () => (/* binding */ FUND_INTERVAL),
/* harmony export */   "COOLDOWN": () => (/* binding */ COOLDOWN),
/* harmony export */   "THRESHOLD": () => (/* binding */ THRESHOLD),
/* harmony export */   "USER_GIVES_UP_AFTER": () => (/* binding */ USER_GIVES_UP_AFTER),
/* harmony export */   "MAX_USERS": () => (/* binding */ MAX_USERS),
/* harmony export */   "MAX_INITIAL": () => (/* binding */ MAX_INITIAL),
/* harmony export */   "format": () => (/* binding */ format),
/* harmony export */   "T": () => (/* binding */ T),
/* harmony export */   "Pool": () => (/* binding */ Pool),
/* harmony export */   "User": () => (/* binding */ User)
/* harmony export */ });
/* harmony import */ var _gruvbox__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./gruvbox */ "./dashboard/gruvbox.ts");

// settings ----------------------------------------------------------------------------------------
const TIME_SCALE = 120, FUND_PORTIONS = 7, DIGITS = 1000000, DIGITS_INV = Math.log10(DIGITS), FUND_PORTION = 2500 * DIGITS, FUND_INTERVAL = 17280 / TIME_SCALE, COOLDOWN = FUND_INTERVAL, THRESHOLD = FUND_INTERVAL, USER_GIVES_UP_AFTER = Infinity, MAX_USERS = 20, MAX_INITIAL = 10000;
const format = {
    integer: (x) => String(x),
    decimal: (x) => (x / DIGITS).toFixed(DIGITS_INV),
    percentage: (x) => `${format.decimal(x)}%`
};
// root of time (warning, singleton!) --------------------------------------------------------------
const T = { T: 0 };
class RPT {
    constructor() {
        this.interval = FUND_INTERVAL;
        this.portion = FUND_PORTION;
        this.remaining = FUND_PORTIONS;
    }
    vest() {
        if (T.T % this.interval == 0) {
            console.info('fund', this.portion, this.remaining);
            if (this.remaining > 0) {
                this.portion;
                this.remaining -= 1;
                return this.portion;
            }
        }
        return 0;
    }
}
class Pool {
    constructor(ui) {
        this.rpt = new RPT();
        this.last_update = 0;
        this.lifetime = 0;
        this.locked = 0;
        this.balance = this.rpt.vest();
        this.claimed = 0;
        this.cooldown = 0;
        this.threshold = 0;
        this.liquid = 0;
        this.ui = ui;
    }
    update() {
        this.balance += this.rpt.vest();
        this.ui.log.now.setValue(T.T);
        this.ui.log.lifetime.setValue(this.lifetime);
        this.ui.log.locked.setValue(this.locked);
        this.ui.log.balance.setValue(format.decimal(this.balance));
        this.ui.log.claimed.setValue(format.decimal(this.claimed));
        this.ui.log.remaining.setValue(this.rpt.remaining);
        this.ui.log.cooldown.setValue(this.cooldown);
        this.ui.log.threshold.setValue(this.threshold);
        this.ui.log.liquid.setValue(format.percentage(this.liquid));
    }
}
class User {
    constructor(ui, pool, name, balance) {
        this.last_update = 0;
        this.lifetime = 0;
        this.locked = 0;
        this.age = 0;
        this.earned = 0;
        this.claimed = 0;
        this.claimable = 0;
        this.cooldown = 0;
        this.waited = 0;
        this.last_claimed = 0;
        this.share = 0;
        this.ui = ui;
        this.pool = pool;
        this.name = name;
        this.balance = balance;
    }
    update() {
        this.ui.table.update(this);
    }
    lock(amount) {
        this.ui.log.add('locks', this.name, amount);
        this.ui.current.add(this);
        this.ui.stacked.add(this);
    }
    retrieve(amount) {
        this.ui.log.add('retrieves', this.name, amount);
        if (this.locked === 0)
            this.ui.current.remove(this);
    }
    claim() {
        throw new Error('not implemented');
    }
    doClaim(reward) {
        console.debug(this.name, 'claim', reward);
        if (reward <= 0)
            return 0;
        if (this.locked === 0)
            return 0;
        if (this.cooldown > 0 || this.age < THRESHOLD)
            return 0;
        if (this.claimed > this.earned) {
            this.ui.log.add('crowded out A', this.name, undefined);
            return 0;
        }
        if (reward > this.pool.balance) {
            this.ui.log.add('crowded out B', this.name, undefined);
            return 0;
        }
        this.pool.balance -= reward;
        this.ui.log.add('claim', this.name, reward);
        console.debug('claimed:', reward);
        return reward;
    }
    colors() {
        return (0,_gruvbox__WEBPACK_IMPORTED_MODULE_0__.COLORS)(this.pool, this);
    }
}


/***/ }),

/***/ "./dashboard/contract_real.ts":
/*!************************************!*\
  !*** ./dashboard/contract_real.ts ***!
  \************************************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

__webpack_require__.r(__webpack_exports__);
/* harmony export */ __webpack_require__.d(__webpack_exports__, {
/* harmony export */   "default": () => (/* binding */ initReal),
/* harmony export */   "RealPool": () => (/* binding */ RealPool),
/* harmony export */   "RealUser": () => (/* binding */ RealUser)
/* harmony export */ });
/* harmony import */ var _helpers__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./helpers */ "./dashboard/helpers.ts");
/* harmony import */ var _contract_base__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ./contract_base */ "./dashboard/contract_base.ts");
/* harmony import */ var _target_web_rewards_js__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! ../target/web/rewards.js */ "./target/web/rewards.js");
var __awaiter = (undefined && undefined.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};



class Rewards {
    constructor() {
        this.index = 0;
        this.contract = new _target_web_rewards_js__WEBPACK_IMPORTED_MODULE_2__.Contract();
        this.debug = false;
    }
    init(msg) {
        this.index += 1;
        this.block = _contract_base__WEBPACK_IMPORTED_MODULE_1__.T.T;
        //if (this.debug) console.debug(`init> ${this.index}`, msg)
        const res = (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.decode)(this.contract.init((0,_helpers__WEBPACK_IMPORTED_MODULE_0__.encode)(msg)));
        //if (this.debug) console.debug(`<init ${this.index}`, res)
        return res;
    }
    query(msg) {
        this.index += 1;
        this.block = _contract_base__WEBPACK_IMPORTED_MODULE_1__.T.T;
        //if (this.debug) console.debug(`query> ${this.index}`, msg)
        const res = (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.decode)(this.contract.query((0,_helpers__WEBPACK_IMPORTED_MODULE_0__.encode)(msg)));
        //if (this.debug) console.debug(`<query ${this.index}`, res)
        return res;
    }
    handle(msg) {
        this.index += 1;
        this.block = _contract_base__WEBPACK_IMPORTED_MODULE_1__.T.T;
        //if (this.debug) console.debug(`handle> ${this.index}`, msg)
        const res = (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.decode)(this.contract.handle((0,_helpers__WEBPACK_IMPORTED_MODULE_0__.encode)(msg)));
        res.log = Object.fromEntries(Object
            .values(res.log)
            .map(({ key, value }) => [key, value]));
        if (Object.keys(res.log).length > 0)
            console.log(res.log);
        //if (this.debug) console.debug(`<handle ${this.index}`, res)
        return res;
    }
    set next_query_response(response) {
        this.contract.next_query_response = (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.encode)(response);
    }
    set sender(address) {
        this.contract.sender = (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.encode)(address);
    }
    set block(height) {
        this.contract.block = BigInt(height);
    }
}
// wasm module load & init -------------------------------------------------------------------------
function initReal() {
    return __awaiter(this, void 0, void 0, function* () {
        // thankfully wasm-pack/wasm-bindgen left an escape hatch
        // because idk wtf is going on with the default loading code
        const url = new URL('rewards_bg.wasm', location.href), res = yield fetch(url.toString()), buf = yield res.arrayBuffer();
        yield (0,_target_web_rewards_js__WEBPACK_IMPORTED_MODULE_2__.default)(buf);
    });
}
// pool api ----------------------------------------------------------------------------------------
class RealPool extends _contract_base__WEBPACK_IMPORTED_MODULE_1__.Pool {
    constructor(ui) {
        super(ui);
        this.contract = new Rewards();
        this.contract.init({
            reward_token: { address: "", code_hash: "" },
            lp_token: { address: "", code_hash: "" },
            viewing_key: "",
            threshold: _contract_base__WEBPACK_IMPORTED_MODULE_1__.THRESHOLD,
            cooldown: _contract_base__WEBPACK_IMPORTED_MODULE_1__.COOLDOWN
        });
        this.ui.log.close.onclick = this.close.bind(this);
    }
    update() {
        this.contract.next_query_response = { balance: { amount: String(this.balance) } };
        const info = this.contract.query({ pool_info: { at: _contract_base__WEBPACK_IMPORTED_MODULE_1__.T.T } }).pool_info;
        //console.log(info)
        this.last_update = info.pool_last_update;
        this.lifetime = info.pool_lifetime;
        this.locked = info.pool_locked;
        this.claimed = info.pool_claimed;
        this.threshold = info.pool_threshold;
        this.cooldown = info.pool_cooldown;
        this.liquid = info.pool_liquid;
        super.update();
    }
    close() {
        this.contract.sender = "";
        this.contract.handle({ close_pool: { message: "pool closed" } });
    }
}
// user api ----------------------------------------------------------------------------------------
class RealUser extends _contract_base__WEBPACK_IMPORTED_MODULE_1__.User {
    constructor(ui, pool, name, balance) {
        super(ui, pool, name, balance);
        this.address = this.name;
        this.contract.sender = this.address;
        this.contract.handle({ set_viewing_key: { key: "" } });
    }
    get contract() {
        return this.pool.contract;
    }
    update() {
        // mock the user's balance - actually stored on this same object
        // because we don't have a snip20 contract to maintain it
        this.contract.next_query_response = { balance: { amount: String(this.pool.balance) } };
        // get the user's info as stored and calculated by the rewards contract
        // presuming the above mock balance
        const info = this.contract.query({ user_info: { at: _contract_base__WEBPACK_IMPORTED_MODULE_1__.T.T, address: this.address, key: "" } }).user_info;
        this.last_update = info.user_last_update;
        this.lifetime = Number(info.user_lifetime);
        this.locked = Number(info.user_locked);
        this.share = Number(info.user_share);
        this.age = Number(info.user_age);
        this.earned = Number(info.user_earned);
        this.claimed = Number(info.user_claimed);
        this.claimable = Number(info.user_claimable);
        this.cooldown = Number(info.user_cooldown);
        super.update();
    }
    lock(amount) {
        this.contract.sender = this.address;
        try {
            //console.debug('lock', amount)
            this.contract.handle({ lock: { amount: String(amount) } });
            super.lock(amount);
        }
        catch (e) {
            //console.error(e)
        }
    }
    retrieve(amount) {
        this.contract.sender = this.address;
        try {
            //console.debug('retrieve', amount)
            this.contract.handle({ retrieve: { amount: String(amount) } });
            super.retrieve(amount);
        }
        catch (e) {
            //console.error(e)
        }
    }
    claim() {
        this.contract.sender = this.address;
        try {
            const result = this.contract.handle({ claim: {} });
            const reward = Number(result.log.reward);
            return this.doClaim(reward);
        }
        catch (e) {
            console.error(e);
            return 0;
        }
    }
}


/***/ }),

/***/ "./dashboard/gruvbox.ts":
/*!******************************!*\
  !*** ./dashboard/gruvbox.ts ***!
  \******************************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

__webpack_require__.r(__webpack_exports__);
/* harmony export */ __webpack_require__.d(__webpack_exports__, {
/* harmony export */   "default": () => (__WEBPACK_DEFAULT_EXPORT__),
/* harmony export */   "COLORS": () => (/* binding */ COLORS)
/* harmony export */ });
/* harmony import */ var _contract_base__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./contract_base */ "./dashboard/contract_base.ts");
// https://git.snoot.club/chee/gruvbox.js/src/branch/master/LICENSE
const dark0Hard = '#1d2021';
const dark0 = '#282828';
const dark0Soft = '#32302f';
const dark1 = '#3c3836';
const dark2 = '#504945';
const dark3 = '#665c54';
const dark4 = '#7c6f64';
const gray245 = '#928374';
const gray244 = '#928374';
const light0Hard = '#f9f5d7';
const light0 = '#fbf1c7';
const light0Soft = '#f2e5bc';
const light1 = '#ebdbb2';
const light2 = '#d5c4a1';
const light3 = '#bdae93';
const light4 = '#a89984';
const brightRed = '#fb4934';
const brightGreen = '#b8bb26';
const brightYellow = '#fabd2f';
const brightBlue = '#83a598';
const brightPurple = '#d3869b';
const brightAqua = '#8ec07c';
const brightOrange = '#fe8019';
const neutralRed = '#cc241d';
const neutralGreen = '#98971a';
const neutralYellow = '#d79921';
const neutralBlue = '#458588';
const neutralPurple = '#b16286';
const neutralAqua = '#689d6a';
const neutralOrange = '#d65d0e';
const fadedRed = '#9d0006';
const fadedGreen = '#79740e';
const fadedYellow = '#b57614';
const fadedBlue = '#076678';
const fadedPurple = '#8f3f71';
const fadedAqua = '#427b58';
const fadedOrange = '#af3a03';
const Gruvbox = {
    dark0Hard,
    dark0Soft,
    dark0,
    dark1,
    dark2,
    dark3,
    dark4,
    dark: {
        hard: dark0Hard,
        soft: dark0Soft,
        0: dark0,
        1: dark1,
        2: dark2,
        3: dark3,
        4: dark4
    },
    gray244,
    gray245,
    gray: {
        244: gray244,
        245: gray245
    },
    light0Hard,
    light0Soft,
    light0,
    light1,
    light2,
    light3,
    light4,
    light: {
        hard: light0Hard,
        soft: light0Soft,
        0: light0,
        1: light1,
        2: light2,
        3: light3,
        4: light4
    },
    brightRed,
    brightGreen,
    brightYellow,
    brightBlue,
    brightPurple,
    brightAqua,
    brightOrange,
    bright: {
        red: brightRed,
        green: brightGreen,
        yellow: brightYellow,
        blue: brightBlue,
        purple: brightPurple,
        aqua: brightAqua,
        orange: brightOrange
    },
    neutralRed,
    neutralGreen,
    neutralYellow,
    neutralBlue,
    neutralPurple,
    neutralAqua,
    neutralOrange,
    neutral: {
        red: neutralRed,
        green: neutralGreen,
        yellow: neutralYellow,
        blue: neutralBlue,
        purple: neutralPurple,
        aqua: neutralAqua,
        orange: neutralOrange
    },
    fadedRed,
    fadedGreen,
    fadedYellow,
    fadedBlue,
    fadedPurple,
    fadedAqua,
    fadedOrange,
    faded: {
        red: fadedRed,
        green: fadedGreen,
        yellow: fadedYellow,
        blue: fadedBlue,
        purple: fadedPurple,
        aqua: fadedAqua,
        orange: fadedOrange
    }
};
/* harmony default export */ const __WEBPACK_DEFAULT_EXPORT__ = (Gruvbox);

const COLORS = Object.assign(function getColor(pool, user) {
    switch (true) {
        case user.age < _contract_base__WEBPACK_IMPORTED_MODULE_0__.THRESHOLD || user.cooldown > 0: // waiting for age threshold
            return COLORS.COOLDOWN;
        case user.claimable > 0 && user.cooldown == 1: // have rewards to claim
            return COLORS.CLAIMING;
        //case user.claimable > 0 && user.cooldown > 0: // just claimed, cooling down
        //return COLORS.ALL_OK
        case user.claimable > pool.balance: // not enough money in pool
            return COLORS.BLOCKED;
        case user.claimed > user.earned: // crowded out
            return COLORS.CROWDED;
        case user.claimable === 0:
            return COLORS.NOTHING;
        default:
            return COLORS.CLAIMABLE;
    }
}, {
    CLAIMABLE: [Gruvbox.fadedAqua, Gruvbox.brightAqua],
    CLAIMING: [Gruvbox.brightAqua, Gruvbox.brightAqua],
    BLOCKED: [Gruvbox.fadedOrange, Gruvbox.brightOrange],
    CROWDED: [Gruvbox.fadedPurple, Gruvbox.brightPurple],
    COOLDOWN: [Gruvbox.fadedBlue, Gruvbox.brightBlue],
    NOTHING: [Gruvbox.dark0, Gruvbox.brightYellow]
});


/***/ }),

/***/ "./dashboard/helpers.ts":
/*!******************************!*\
  !*** ./dashboard/helpers.ts ***!
  \******************************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

__webpack_require__.r(__webpack_exports__);
/* harmony export */ __webpack_require__.d(__webpack_exports__, {
/* harmony export */   "random": () => (/* binding */ random),
/* harmony export */   "pickRandom": () => (/* binding */ pickRandom),
/* harmony export */   "throttle": () => (/* binding */ throttle),
/* harmony export */   "after": () => (/* binding */ after),
/* harmony export */   "h": () => (/* binding */ h),
/* harmony export */   "append": () => (/* binding */ append),
/* harmony export */   "prepend": () => (/* binding */ prepend),
/* harmony export */   "encode": () => (/* binding */ encode),
/* harmony export */   "decode": () => (/* binding */ decode)
/* harmony export */ });
// randomness helpers ------------------------------------------------------------------------------
const random = (max) => Math.floor(Math.random() * max);
const pickRandom = (x) => x[random(x.length)];
// timing helpers ----------------------------------------------------------------------------------
function throttle(t, fn) {
    // todo replacing t with a function allows for implementing exponential backoff
    let timeout;
    return function throttled(...args) {
        return new Promise(resolve => {
            if (timeout)
                clearTimeout(timeout);
            timeout = after(t, () => resolve(fn(...args)));
        });
    };
}
function after(t, fn) {
    return setTimeout(fn, t);
}
// DOM helpers -------------------------------------------------------------------------------------
function h(element, attributes = {}, ...content) {
    const el = Object.assign(document.createElement(element), attributes);
    for (const el2 of content)
        el.appendChild(el2);
    return el;
}
function append(parent, child) {
    return parent.appendChild(child);
}
function prepend(parent, child) {
    return parent.insertBefore(child, parent.firstChild);
}
// convert from string to Utf8Array ----------------------------------------------------------------
const enc = new TextEncoder();
const encode = (x) => enc.encode(JSON.stringify(x));
const dec = new TextDecoder();
const decode = (x) => JSON.parse(dec.decode(x.buffer));


/***/ }),

/***/ "./dashboard/widgets.ts":
/*!******************************!*\
  !*** ./dashboard/widgets.ts ***!
  \******************************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

__webpack_require__.r(__webpack_exports__);
/* harmony export */ __webpack_require__.d(__webpack_exports__, {
/* harmony export */   "NO_HISTORY": () => (/* binding */ NO_HISTORY),
/* harmony export */   "NO_TABLE": () => (/* binding */ NO_TABLE),
/* harmony export */   "Field": () => (/* binding */ Field),
/* harmony export */   "Log": () => (/* binding */ Log),
/* harmony export */   "Table": () => (/* binding */ Table),
/* harmony export */   "PieChart": () => (/* binding */ PieChart),
/* harmony export */   "StackedPieChart": () => (/* binding */ StackedPieChart)
/* harmony export */ });
/* harmony import */ var _helpers__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./helpers */ "./dashboard/helpers.ts");
/* harmony import */ var _contract_base__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ./contract_base */ "./dashboard/contract_base.ts");


// killswitches for gui components -----------------------------------------------------------------
const NO_HISTORY = true;
const NO_TABLE = false;
// Label + value
class Field {
    constructor(name, value) {
        this.root = (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('div', { className: 'field' });
        this.label = (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.append)(this.root, (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('label'));
        this.value = (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.append)(this.root, (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('div'));
        this.label.textContent = name;
        this.value.textContent = String(value);
    }
    append(parent) {
        parent.appendChild(this.root);
        return this;
    }
    setValue(value) {
        this.value.textContent = String(value);
    }
}
// global values + log of all modeled events -------------------------------------------------------
class Log {
    constructor() {
        this.root = (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('div', { className: 'history' });
        this.body = (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.append)(this.root, (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('ol'));
        this.now = new Field('block').append(this.root);
        this.locked = new Field('liquidity now in pool').append(this.root);
        this.lifetime = new Field('all liquidity ever in pool').append(this.root);
        this.balance = new Field('available reward balance').append(this.root);
        this.claimed = new Field('rewards claimed by users').append(this.root);
        this.remaining = new Field('remaining funding portions').append(this.root);
        this.threshold = new Field('initial age threshold').append(this.root);
        this.cooldown = new Field('cooldown after claim').append(this.root);
        this.liquid = new Field('pool liquidity ratio').append(this.root);
        this.close = (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.append)(this.root, (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('button', { textContent: 'close pool' }));
    }
    add(event, name, amount) {
        if (NO_HISTORY)
            return;
        if (amount) {
            (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.prepend)(this.body, (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('div', { innerHTML: `<b>${name}</b> ${event} ${amount}LP` }));
        }
        else {
            (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.prepend)(this.body, (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('div', { innerHTML: `<b>${name}</b> ${event}` }));
        }
    }
}
class Table {
    constructor() {
        this.rows = {};
        this.root = document.createElement('table');
        if (NO_TABLE)
            return;
    }
    init(users) {
        (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.append)(this.root, (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('thead', {}, (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('th', { textContent: 'name' }), (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('th', { textContent: 'last_update' }), (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('th', { textContent: 'age' }), (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('th', { textContent: 'locked' }), (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('th', { textContent: 'lifetime' }), (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('th', { textContent: 'share' }), (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('th', { textContent: 'earned' }), (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('th', { textContent: 'claimed' }), (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('th', { textContent: 'claimable' }), (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('th', { textContent: 'cooldown' })));
        for (const name of Object.keys(users)) {
            this.addRow(name, users[name]);
        }
    }
    addRow(name, user) {
        if (NO_TABLE)
            return;
        const row = (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.append)(this.root, (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('tr'));
        const locked = (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('td', { className: 'locked' }), lockedMinus = (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.append)(locked, (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('button', {
            textContent: '-',
            onclick: () => user.retrieve(100)
        })), lockedValue = (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.append)(locked, (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('span', {
            textContent: ''
        })), lockedPlus = (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.append)(locked, (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('button', {
            textContent: '+',
            onclick: () => user.lock(100)
        }));
        const rows = this.rows[name] = {
            name: (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.append)(row, (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('td', { style: 'font-weight:bold', textContent: name })),
            last_update: (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.append)(row, (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('td')),
            age: (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.append)(row, (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('td')),
            locked: (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.append)(row, locked),
            lockedMinus, lockedValue, lockedPlus,
            lifetime: (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.append)(row, (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('td')),
            share: (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.append)(row, (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('td')),
            earned: (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.append)(row, (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('td')),
            claimed: (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.append)(row, (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('td')),
            claimable: (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.append)(row, (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('td', { className: 'claimable', onclick: () => { user.claim(); } })),
            cooldown: (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.append)(row, (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('td')),
        };
        rows.claimable.style.fontWeight = 'bold';
        (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.append)(this.root, row);
        return rows;
    }
    update(user) {
        if (NO_TABLE)
            return;
        this.rows[user.name].last_update.textContent =
            _contract_base__WEBPACK_IMPORTED_MODULE_1__.format.integer(user.last_update);
        this.rows[user.name].lockedValue.textContent =
            _contract_base__WEBPACK_IMPORTED_MODULE_1__.format.integer(user.locked);
        this.rows[user.name].lifetime.textContent =
            _contract_base__WEBPACK_IMPORTED_MODULE_1__.format.integer(user.lifetime);
        this.rows[user.name].share.textContent =
            _contract_base__WEBPACK_IMPORTED_MODULE_1__.format.percentage(user.share);
        this.rows[user.name].age.textContent =
            _contract_base__WEBPACK_IMPORTED_MODULE_1__.format.integer(user.age);
        this.rows[user.name].earned.textContent =
            _contract_base__WEBPACK_IMPORTED_MODULE_1__.format.decimal(user.earned);
        this.rows[user.name].claimed.textContent =
            _contract_base__WEBPACK_IMPORTED_MODULE_1__.format.decimal(user.claimed);
        this.rows[user.name].claimable.textContent =
            _contract_base__WEBPACK_IMPORTED_MODULE_1__.format.decimal(user.claimable);
        this.rows[user.name].cooldown.textContent =
            _contract_base__WEBPACK_IMPORTED_MODULE_1__.format.integer(user.cooldown);
        const [fill, stroke] = user.colors();
        this.rows[user.name].earned.style.backgroundColor =
            this.rows[user.name].claimed.style.backgroundColor =
                this.rows[user.name].claimable.style.backgroundColor =
                    fill;
        this.rows[user.name].claimable.style.color =
            stroke;
    }
}
class PieChart {
    constructor(_name, field) {
        this.users = {};
        this.total = 0;
        this.field = field;
        this.root = (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('div', { className: `pie ${field}` });
        this.canvas = (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.append)(this.root, (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('canvas', { width: 1, height: 1 }));
    }
    add(user) {
        this.users[user.name] = user;
    }
    remove(user) {
        delete this.users[user.name];
    }
    resize() {
        this.canvas.width = this.canvas.height = 1;
        const size = Math.min(this.root.offsetWidth, this.root.offsetHeight);
        this.canvas.width = this.canvas.height = size;
        this.render();
    }
    render() {
        requestAnimationFrame(() => {
            // extract needed datum from user list
            // and sum the total
            const values = {};
            let total = 0;
            for (const user of Object.values(this.users)) {
                const value = user[this.field];
                if (value) {
                    total += value;
                    values[user.name] = value;
                }
            }
            if (total === 0)
                return;
            // prepare canvas
            const { width, height } = this.canvas;
            const context = this.canvas.getContext('2d');
            // clear
            context.fillStyle = '#282828';
            context.fillRect(1, 1, width - 2, height - 2);
            // define center
            const centerX = width / 2;
            const centerY = height / 2;
            const radius = centerX * 0.95;
            // loop over segments
            let start = 0;
            for (const name of Object.keys(this.users).sort()) {
                const value = values[name];
                if (value) {
                    const portion = value / total;
                    const end = start + (2 * portion);
                    context.beginPath();
                    context.moveTo(centerX, centerY);
                    context.arc(centerX, centerY, radius, start * Math.PI, end * Math.PI);
                    //context.moveTo(centerX, centerY)
                    const [fillStyle, strokeStyle] = this.users[name].colors();
                    context.fillStyle = fillStyle;
                    context.lineWidth = 0.8;
                    context.strokeStyle = strokeStyle; // '#000'//rgba(255,255,255,0.5)'
                    context.fill();
                    context.stroke();
                    start = end;
                }
            }
        });
    }
}
class StackedPieChart {
    constructor() {
        this.users = {};
        this.root = (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('div', { className: `pie stacked` });
        this.canvas = (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.append)(this.root, (0,_helpers__WEBPACK_IMPORTED_MODULE_0__.h)('canvas', { width: 1, height: 1 }));
    }
    add(user) {
        this.users[user.name] = user;
    }
    remove(user) {
        delete this.users[user.name];
    }
    resize() {
        this.canvas.width = this.canvas.height = 1;
        const size = Math.min(this.root.offsetWidth, this.root.offsetHeight);
        this.canvas.width = this.canvas.height = size;
        this.render();
    }
    render() {
        requestAnimationFrame(() => {
            // extract needed datum from user list
            // and sum the total
            let total = 0;
            for (const user of Object.values(this.users)) {
                total += user.lifetime;
            }
            if (total === 0)
                return;
            // prepare canvas
            const { width, height } = this.canvas;
            const context = this.canvas.getContext('2d');
            // clear
            context.fillStyle = '#282828';
            context.fillRect(1, 1, width - 2, height - 2);
            // define center
            const centerX = width / 2;
            const centerY = height / 2;
            const radius = centerX * 0.95;
            // loop over segments
            let start = 0;
            for (const name of Object.keys(this.users).sort()) {
                const user = this.users[name];
                if (user.lifetime === 0)
                    continue;
                const portion = user.lifetime / total;
                const end = start + (2 * portion);
                context.beginPath();
                context.moveTo(centerX, centerY);
                context.arc(centerX, centerY, radius, start * Math.PI, end * Math.PI);
                //context.moveTo(centerX, centerY)
                const [fillStyle, strokeStyle] = user.colors();
                context.fillStyle = fillStyle;
                context.strokeStyle = strokeStyle; //'#000'//'rgba(255,255,255,0.5)'
                //context.strokeStyle = fillStyle//strokeStyle
                context.lineWidth = 0.8;
                context.fill();
                context.stroke();
                start = end;
            }
        });
    }
}


/***/ })

/******/ 	});
/************************************************************************/
/******/ 	// The module cache
/******/ 	var __webpack_module_cache__ = {};
/******/ 	
/******/ 	// The require function
/******/ 	function __webpack_require__(moduleId) {
/******/ 		// Check if module is in cache
/******/ 		var cachedModule = __webpack_module_cache__[moduleId];
/******/ 		if (cachedModule !== undefined) {
/******/ 			return cachedModule.exports;
/******/ 		}
/******/ 		// Create a new module (and put it into the cache)
/******/ 		var module = __webpack_module_cache__[moduleId] = {
/******/ 			id: moduleId,
/******/ 			// no module.loaded needed
/******/ 			exports: {}
/******/ 		};
/******/ 	
/******/ 		// Execute the module function
/******/ 		__webpack_modules__[moduleId](module, module.exports, __webpack_require__);
/******/ 	
/******/ 		// Return the exports of the module
/******/ 		return module.exports;
/******/ 	}
/******/ 	
/******/ 	// expose the modules object (__webpack_modules__)
/******/ 	__webpack_require__.m = __webpack_modules__;
/******/ 	
/************************************************************************/
/******/ 	/* webpack/runtime/compat get default export */
/******/ 	(() => {
/******/ 		// getDefaultExport function for compatibility with non-harmony modules
/******/ 		__webpack_require__.n = (module) => {
/******/ 			var getter = module && module.__esModule ?
/******/ 				() => (module['default']) :
/******/ 				() => (module);
/******/ 			__webpack_require__.d(getter, { a: getter });
/******/ 			return getter;
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/define property getters */
/******/ 	(() => {
/******/ 		// define getter functions for harmony exports
/******/ 		__webpack_require__.d = (exports, definition) => {
/******/ 			for(var key in definition) {
/******/ 				if(__webpack_require__.o(definition, key) && !__webpack_require__.o(exports, key)) {
/******/ 					Object.defineProperty(exports, key, { enumerable: true, get: definition[key] });
/******/ 				}
/******/ 			}
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/global */
/******/ 	(() => {
/******/ 		__webpack_require__.g = (function() {
/******/ 			if (typeof globalThis === 'object') return globalThis;
/******/ 			try {
/******/ 				return this || new Function('return this')();
/******/ 			} catch (e) {
/******/ 				if (typeof window === 'object') return window;
/******/ 			}
/******/ 		})();
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/hasOwnProperty shorthand */
/******/ 	(() => {
/******/ 		__webpack_require__.o = (obj, prop) => (Object.prototype.hasOwnProperty.call(obj, prop))
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/make namespace object */
/******/ 	(() => {
/******/ 		// define __esModule on exports
/******/ 		__webpack_require__.r = (exports) => {
/******/ 			if(typeof Symbol !== 'undefined' && Symbol.toStringTag) {
/******/ 				Object.defineProperty(exports, Symbol.toStringTag, { value: 'Module' });
/******/ 			}
/******/ 			Object.defineProperty(exports, '__esModule', { value: true });
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/publicPath */
/******/ 	(() => {
/******/ 		var scriptUrl;
/******/ 		if (__webpack_require__.g.importScripts) scriptUrl = __webpack_require__.g.location + "";
/******/ 		var document = __webpack_require__.g.document;
/******/ 		if (!scriptUrl && document) {
/******/ 			if (document.currentScript)
/******/ 				scriptUrl = document.currentScript.src
/******/ 			if (!scriptUrl) {
/******/ 				var scripts = document.getElementsByTagName("script");
/******/ 				if(scripts.length) scriptUrl = scripts[scripts.length - 1].src
/******/ 			}
/******/ 		}
/******/ 		// When supporting browsers where an automatic publicPath is not supported you must specify an output.publicPath manually via configuration
/******/ 		// or pass an empty string ("") and set the __webpack_public_path__ variable from your code to use your own logic.
/******/ 		if (!scriptUrl) throw new Error("Automatic publicPath is not supported in this browser");
/******/ 		scriptUrl = scriptUrl.replace(/#.*$/, "").replace(/\?.*$/, "").replace(/\/[^\/]+$/, "/");
/******/ 		__webpack_require__.p = scriptUrl;
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/jsonp chunk loading */
/******/ 	(() => {
/******/ 		__webpack_require__.b = document.baseURI || self.location.href;
/******/ 		
/******/ 		// object to store loaded and loading chunks
/******/ 		// undefined = chunk not loaded, null = chunk preloaded/prefetched
/******/ 		// [resolve, reject, Promise] = chunk loading, 0 = chunk loaded
/******/ 		var installedChunks = {
/******/ 			"main": 0
/******/ 		};
/******/ 		
/******/ 		// no chunk on demand loading
/******/ 		
/******/ 		// no prefetching
/******/ 		
/******/ 		// no preloaded
/******/ 		
/******/ 		// no HMR
/******/ 		
/******/ 		// no HMR manifest
/******/ 		
/******/ 		// no on chunks loaded
/******/ 		
/******/ 		// no jsonp function
/******/ 	})();
/******/ 	
/************************************************************************/
var __webpack_exports__ = {};
// This entry need to be wrapped in an IIFE because it need to be isolated against other modules in the chunk.
(() => {
/*!******************************!*\
  !*** ./rewards_dashboard.ts ***!
  \******************************/
__webpack_require__.r(__webpack_exports__);
/* harmony import */ var _dashboard_style_css__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./dashboard/style.css */ "./dashboard/style.css");
/* harmony import */ var _dashboard_widgets__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ./dashboard/widgets */ "./dashboard/widgets.ts");
/* harmony import */ var _dashboard_contract_base__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! ./dashboard/contract_base */ "./dashboard/contract_base.ts");
/* harmony import */ var _dashboard_contract_real__WEBPACK_IMPORTED_MODULE_3__ = __webpack_require__(/*! ./dashboard/contract_real */ "./dashboard/contract_real.ts");
/* harmony import */ var _dashboard_helpers__WEBPACK_IMPORTED_MODULE_4__ = __webpack_require__(/*! ./dashboard/helpers */ "./dashboard/helpers.ts");





//import initMock from './dashboard/contract_mock'

document.body.innerHTML = '<center>loading</center>';
// settings ----------------------------------------------------------------------------------------
const UPDATE_INTERVAL = 1;
const AUTO_CLAIM = false;
const AUTO_LOCK_UNLOCK = false;
(0,_dashboard_contract_real__WEBPACK_IMPORTED_MODULE_3__.default)().then(() => {
    document.body.onclick = () => {
        document.body.innerHTML = '';
        document.body.onclick = null;
        start();
    };
    document.body.innerHTML = '<center>click to start</center>';
});
function start() {
    // create the dashboard --------------------------------------------------------------------------
    const ui = {
        log: new _dashboard_widgets__WEBPACK_IMPORTED_MODULE_1__.Log(),
        table: new _dashboard_widgets__WEBPACK_IMPORTED_MODULE_1__.Table(),
        current: new _dashboard_widgets__WEBPACK_IMPORTED_MODULE_1__.PieChart('Current amounts locked', 'locked'),
        stacked: new _dashboard_widgets__WEBPACK_IMPORTED_MODULE_1__.StackedPieChart()
    };
    // create a pool and some of test users with random balances -------------------------------------
    const pool = new _dashboard_contract_real__WEBPACK_IMPORTED_MODULE_3__.RealPool(ui);
    const users = {};
    for (let i = 0; i < _dashboard_contract_base__WEBPACK_IMPORTED_MODULE_2__.MAX_USERS; i++) {
        const name = `User${i}`;
        const balance = Math.floor(Math.random() * _dashboard_contract_base__WEBPACK_IMPORTED_MODULE_2__.MAX_INITIAL);
        users[name] = new _dashboard_contract_real__WEBPACK_IMPORTED_MODULE_3__.RealUser(ui, pool, name, balance);
    }
    // add components --------------------------------------------------------------------------------
    for (const el of Object.values(ui)) {
        (0,_dashboard_helpers__WEBPACK_IMPORTED_MODULE_4__.append)(document.body, el.root);
    }
    // create dom elements for all users - then only update the content ------------------------------
    ui.table.init(users);
    // add resize handler ----------------------------------------------------------------------------
    resize();
    window.addEventListener('resize', (0,_dashboard_helpers__WEBPACK_IMPORTED_MODULE_4__.throttle)(100, resize));
    // start updating --------------------------------------------------------------------------------
    update();
    function update() {
        // advance time --------------------------------------------------------------------------------
        _dashboard_contract_base__WEBPACK_IMPORTED_MODULE_2__.T.T++;
        pool.contract.block = _dashboard_contract_base__WEBPACK_IMPORTED_MODULE_2__.T.T;
        // periodically fund pool and increment its lifetime -------------------------------------------
        pool.update();
        // increment lifetimes and ages; collect eligible claimants ------------------------------------
        const eligible = [];
        for (const user of Object.values(users)) {
            user.update();
            if (user.claimable > 0)
                eligible.push(user);
        }
        // perform random lock/retrieve from random account for random amount --------------------------
        if (AUTO_LOCK_UNLOCK) {
            const user = (0,_dashboard_helpers__WEBPACK_IMPORTED_MODULE_4__.pickRandom)(Object.values(users));
            (0,_dashboard_helpers__WEBPACK_IMPORTED_MODULE_4__.pickRandom)([
                (amount) => user.lock(amount),
                (amount) => user.retrieve(amount)
            ])((0,_dashboard_helpers__WEBPACK_IMPORTED_MODULE_4__.random)(user.balance));
        }
        // perform random claim ------------------------------------------------------------------------
        if (AUTO_CLAIM && eligible.length > 0) {
            const claimant = (0,_dashboard_helpers__WEBPACK_IMPORTED_MODULE_4__.pickRandom)(eligible);
            claimant.claim();
        }
        // update charts -------------------------------------------------------------------------------
        for (const chart of [ui.current, ui.stacked]) {
            chart.render();
        }
        // rinse and repeat ----------------------------------------------------------------------------
        (0,_dashboard_helpers__WEBPACK_IMPORTED_MODULE_4__.after)(UPDATE_INTERVAL, update);
    }
    // resize handler --------------------------------------------------------------------------------
    function resize() {
        ui.current.resize();
        ui.stacked.resize();
    }
}

})();

/******/ })()
;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIndlYnBhY2s6Ly8vLi90YXJnZXQvd2ViL3Jld2FyZHMuanMiLCJ3ZWJwYWNrOi8vLy4vZGFzaGJvYXJkL3N0eWxlLmNzcyIsIndlYnBhY2s6Ly8vLi4vLi4vbm9kZV9tb2R1bGVzL2Nzcy1sb2FkZXIvZGlzdC9ydW50aW1lL2FwaS5qcyIsIndlYnBhY2s6Ly8vLi4vLi4vbm9kZV9tb2R1bGVzL2Nzcy1sb2FkZXIvZGlzdC9ydW50aW1lL2Nzc1dpdGhNYXBwaW5nVG9TdHJpbmcuanMiLCJ3ZWJwYWNrOi8vLy4vZGFzaGJvYXJkL3N0eWxlLmNzcz8xMTBmIiwid2VicGFjazovLy8uLi8uLi9ub2RlX21vZHVsZXMvc3R5bGUtbG9hZGVyL2Rpc3QvcnVudGltZS9pbmplY3RTdHlsZXNJbnRvU3R5bGVUYWcuanMiLCJ3ZWJwYWNrOi8vLy4uLy4uL25vZGVfbW9kdWxlcy9zdHlsZS1sb2FkZXIvZGlzdC9ydW50aW1lL2luc2VydEJ5U2VsZWN0b3IuanMiLCJ3ZWJwYWNrOi8vLy4uLy4uL25vZGVfbW9kdWxlcy9zdHlsZS1sb2FkZXIvZGlzdC9ydW50aW1lL2luc2VydFN0eWxlRWxlbWVudC5qcyIsIndlYnBhY2s6Ly8vLi4vLi4vbm9kZV9tb2R1bGVzL3N0eWxlLWxvYWRlci9kaXN0L3J1bnRpbWUvc2V0QXR0cmlidXRlc1dpdGhvdXRBdHRyaWJ1dGVzLmpzIiwid2VicGFjazovLy8uLi8uLi9ub2RlX21vZHVsZXMvc3R5bGUtbG9hZGVyL2Rpc3QvcnVudGltZS9zdHlsZURvbUFQSS5qcyIsIndlYnBhY2s6Ly8vLi4vLi4vbm9kZV9tb2R1bGVzL3N0eWxlLWxvYWRlci9kaXN0L3J1bnRpbWUvc3R5bGVUYWdUcmFuc2Zvcm0uanMiLCJ3ZWJwYWNrOi8vLy4vZGFzaGJvYXJkL2NvbnRyYWN0X2Jhc2UudHMiLCJ3ZWJwYWNrOi8vLy4vZGFzaGJvYXJkL2NvbnRyYWN0X3JlYWwudHMiLCJ3ZWJwYWNrOi8vLy4vZGFzaGJvYXJkL2dydXZib3gudHMiLCJ3ZWJwYWNrOi8vLy4vZGFzaGJvYXJkL2hlbHBlcnMudHMiLCJ3ZWJwYWNrOi8vLy4vZGFzaGJvYXJkL3dpZGdldHMudHMiLCJ3ZWJwYWNrOi8vL3dlYnBhY2svYm9vdHN0cmFwIiwid2VicGFjazovLy93ZWJwYWNrL3J1bnRpbWUvY29tcGF0IGdldCBkZWZhdWx0IGV4cG9ydCIsIndlYnBhY2s6Ly8vd2VicGFjay9ydW50aW1lL2RlZmluZSBwcm9wZXJ0eSBnZXR0ZXJzIiwid2VicGFjazovLy93ZWJwYWNrL3J1bnRpbWUvZ2xvYmFsIiwid2VicGFjazovLy93ZWJwYWNrL3J1bnRpbWUvaGFzT3duUHJvcGVydHkgc2hvcnRoYW5kIiwid2VicGFjazovLy93ZWJwYWNrL3J1bnRpbWUvbWFrZSBuYW1lc3BhY2Ugb2JqZWN0Iiwid2VicGFjazovLy93ZWJwYWNrL3J1bnRpbWUvcHVibGljUGF0aCIsIndlYnBhY2s6Ly8vd2VicGFjay9ydW50aW1lL2pzb25wIGNodW5rIGxvYWRpbmciLCJ3ZWJwYWNrOi8vLy4vcmV3YXJkc19kYXNoYm9hcmQudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6Ijs7Ozs7Ozs7Ozs7Ozs7OztBQUNBOztBQUVBLGtEQUFrRCwrQkFBK0I7O0FBRWpGOztBQUVBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBOztBQUVBO0FBQ0E7QUFDQTs7QUFFQTs7QUFFQTs7QUFFQTs7QUFFQTtBQUNBO0FBQ0E7QUFDQTs7QUFFQTs7QUFFQTtBQUNBO0FBQ0E7O0FBRUEseUJBQXlCLGtCQUFrQjs7QUFFM0M7QUFDQTtBQUNBO0FBQ0E7QUFDQTs7QUFFQTtBQUNBO0FBQ0E7QUFDQTtBQUNBOztBQUVBO0FBQ0E7QUFDQTs7QUFFQTs7QUFFQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7O0FBRUE7O0FBRUE7O0FBRUE7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7O0FBRUE7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNPOztBQUVQO0FBQ0E7QUFDQTs7QUFFQTtBQUNBOztBQUVBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7O0FBRUE7QUFDQTtBQUNBOztBQUVBO0FBQ0E7QUFDQTs7QUFFQTtBQUNBOztBQUVBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQSxjQUFjLFdBQVc7QUFDekI7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0EsY0FBYyxPQUFPO0FBQ3JCO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0EsZ0JBQWdCO0FBQ2hCO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0EsU0FBUztBQUNUO0FBQ0E7QUFDQTtBQUNBO0FBQ0EsY0FBYyxXQUFXO0FBQ3pCO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBLGNBQWMsV0FBVztBQUN6QixnQkFBZ0I7QUFDaEI7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBLFNBQVM7QUFDVDtBQUNBO0FBQ0E7QUFDQTtBQUNBLGNBQWMsV0FBVztBQUN6QixnQkFBZ0I7QUFDaEI7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBLFNBQVM7QUFDVDtBQUNBO0FBQ0E7QUFDQTtBQUNBLGNBQWMsV0FBVztBQUN6QixnQkFBZ0I7QUFDaEI7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBLFNBQVM7QUFDVDtBQUNBO0FBQ0E7QUFDQTs7QUFFQTtBQUNBO0FBQ0E7QUFDQTtBQUNBOztBQUVBLGFBQWE7QUFDYjtBQUNBOztBQUVBLGlCQUFpQjtBQUNqQjtBQUNBO0FBQ0E7QUFDQTs7QUFFQTtBQUNBOztBQUVBLEtBQUs7QUFDTDs7QUFFQTtBQUNBLG9CQUFvQjs7QUFFcEIsU0FBUztBQUNUO0FBQ0E7QUFDQTtBQUNBOztBQUVBO0FBQ0E7QUFDQSx3QkFBd0Isb0hBQWtDO0FBQzFEO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBOztBQUVBO0FBQ0E7QUFDQTs7OztBQUlBLFdBQVcsbUJBQW1COztBQUU5QjtBQUNBOztBQUVBO0FBQ0E7O0FBRUEsaUVBQWUsSUFBSSxFQUFDOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7QUNyU3BCO0FBQzRIO0FBQzdCO0FBQy9GLDhCQUE4QixtRkFBMkIsQ0FBQyx3R0FBcUM7QUFDL0Y7QUFDQSw2Q0FBNkMsMkJBQTJCLGNBQWMsZUFBZSxvQkFBb0IscUJBQXFCLEdBQUcsZ0JBQWdCLGtCQUFrQixlQUFlLGVBQWUsNEJBQTRCLEdBQUcsVUFBVSx3QkFBd0Isd0JBQXdCLHFCQUFxQiwrQ0FBK0Msa0NBQWtDLHNDQUFzQyxtQ0FBbUMsUUFBUSxzQkFBc0IsY0FBYyxHQUFHLE1BQU0sd0JBQXdCLHNCQUFzQix3QkFBd0IsR0FBRyxvQkFBb0IsbUJBQW1CLHVCQUF1Qix5QkFBeUIsS0FBSyxVQUFVLHFCQUFxQix5QkFBeUIseUJBQXlCLEdBQUcsZ0JBQWdCLHlCQUF5Qix5QkFBeUIsR0FBRyxhQUFhLDRCQUE0QixLQUFLLGlCQUFpQixLQUFLLFdBQVcsc0JBQXNCLHlCQUF5Qix1QkFBdUIsd0JBQXdCLDhCQUE4QixHQUFHLFFBQVEscUJBQXFCLHVCQUF1QixHQUFHLHNCQUFzQiw0QkFBNEIsb0JBQW9CLCtCQUErQixHQUFHLFNBQVMsd0JBQXdCLEdBQUcsTUFBTSxvQkFBb0IsR0FBRyxjQUFjLHNCQUFzQixvQkFBb0IseUJBQXlCLGlCQUFpQix3QkFBd0IsbUJBQW1CLEdBQUcsZ0JBQWdCLHFCQUFxQixHQUFHLGtCQUFrQiwyQkFBMkIsZ0JBQWdCLGlCQUFpQixrQkFBa0IscUJBQXFCLE9BQU8sY0FBYyx5QkFBeUIsdUJBQXVCLHNCQUFzQixvQkFBb0Isa0JBQWtCLHdCQUF3Qiw0QkFBNEIsb0JBQW9CLEdBQUcsWUFBWSx5QkFBeUIsNkNBQTZDLHdCQUF3QixHQUFHLGdCQUFnQixzQkFBc0IsR0FBRyxnQkFBZ0Isd0JBQXdCLGVBQWUsa0JBQWtCLG1DQUFtQyxrQ0FBa0Msa0JBQWtCLG9CQUFvQixHQUFHLFNBQVMsc0ZBQXNGLFlBQVksV0FBVyxVQUFVLFVBQVUsWUFBWSxPQUFPLEtBQUssVUFBVSxVQUFVLFVBQVUsWUFBWSxPQUFPLEtBQUssWUFBWSxhQUFhLGFBQWEsYUFBYSxNQUFNLE1BQU0sWUFBWSxjQUFjLE1BQU0sWUFBWSxXQUFXLEtBQUssS0FBSyxZQUFZLGFBQWEsYUFBYSxPQUFPLEtBQUssWUFBWSxhQUFhLGFBQWEsT0FBTyxLQUFLLFlBQVksYUFBYSxhQUFhLE1BQU0sS0FBSyxZQUFZLGFBQWEsTUFBTSxLQUFLLFlBQVksTUFBTSxLQUFLLE1BQU0sS0FBSyxZQUFZLGFBQWEsYUFBYSxhQUFhLGFBQWEsT0FBTyxLQUFLLFlBQVksYUFBYSxNQUFNLEtBQUssWUFBWSxXQUFXLFlBQVksTUFBTSxLQUFLLFlBQVksTUFBTSxLQUFLLFVBQVUsT0FBTyxLQUFLLFlBQVksV0FBVyxZQUFZLFdBQVcsWUFBWSxXQUFXLE9BQU8sS0FBSyxZQUFZLE9BQU8sVUFBVSxZQUFZLFdBQVcsVUFBVSxVQUFVLFlBQVksWUFBWSxLQUFLLFlBQVksYUFBYSxhQUFhLFdBQVcsVUFBVSxZQUFZLGFBQWEsV0FBVyxPQUFPLEtBQUssWUFBWSxhQUFhLGFBQWEsTUFBTSxLQUFLLFlBQVksTUFBTSxLQUFLLEtBQUssTUFBTSxLQUFLLFVBQVUsWUFBWSxNQUFNLEtBQUssS0FBSyxVQUFVLDZCQUE2QiwyQkFBMkIsY0FBYyxlQUFlLG9CQUFvQixxQkFBcUIsR0FBRyxnQkFBZ0Isa0JBQWtCLGVBQWUsZUFBZSw0QkFBNEIsR0FBRyxVQUFVLHdCQUF3Qix3QkFBd0IscUJBQXFCLCtDQUErQyxrQ0FBa0Msc0NBQXNDLG1DQUFtQyxRQUFRLHNCQUFzQixjQUFjLEdBQUcsTUFBTSx3QkFBd0Isc0JBQXNCLHdCQUF3QixHQUFHLG9CQUFvQixtQkFBbUIsdUJBQXVCLHlCQUF5QixLQUFLLFVBQVUscUJBQXFCLHlCQUF5Qix5QkFBeUIsR0FBRyxnQkFBZ0IseUJBQXlCLHlCQUF5QixHQUFHLGFBQWEsNEJBQTRCLEtBQUssaUJBQWlCLEtBQUssV0FBVyxzQkFBc0IseUJBQXlCLHVCQUF1Qix3QkFBd0IsOEJBQThCLEdBQUcsUUFBUSxxQkFBcUIsdUJBQXVCLEdBQUcsc0JBQXNCLDRCQUE0QixvQkFBb0IsK0JBQStCLEdBQUcsU0FBUyx3QkFBd0IsR0FBRyxNQUFNLG9CQUFvQixHQUFHLGNBQWMsc0JBQXNCLG9CQUFvQix5QkFBeUIsaUJBQWlCLHdCQUF3QixtQkFBbUIsR0FBRyxnQkFBZ0IscUJBQXFCLEdBQUcsa0JBQWtCLDJCQUEyQixnQkFBZ0IsaUJBQWlCLGtCQUFrQixxQkFBcUIsT0FBTyxjQUFjLHlCQUF5Qix1QkFBdUIsc0JBQXNCLG9CQUFvQixrQkFBa0Isd0JBQXdCLDRCQUE0QixvQkFBb0IsR0FBRyxZQUFZLHlCQUF5Qiw2Q0FBNkMsd0JBQXdCLEdBQUcsZ0JBQWdCLHNCQUFzQixHQUFHLGdCQUFnQix3QkFBd0IsZUFBZSxrQkFBa0IsbUNBQW1DLGtDQUFrQyxrQkFBa0Isb0JBQW9CLEdBQUcscUJBQXFCO0FBQzV5SztBQUNBLGlFQUFlLHVCQUF1QixFQUFDOzs7Ozs7Ozs7OztBQ1AxQjs7QUFFYjtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBLGdCQUFnQjs7QUFFaEI7QUFDQTtBQUNBOztBQUVBO0FBQ0EsNENBQTRDLHFCQUFxQjtBQUNqRTs7QUFFQTtBQUNBLEtBQUs7QUFDTCxJQUFJO0FBQ0o7OztBQUdBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7O0FBRUE7O0FBRUE7QUFDQSxxQkFBcUIsaUJBQWlCO0FBQ3RDO0FBQ0E7O0FBRUE7QUFDQTtBQUNBO0FBQ0E7QUFDQTs7QUFFQSxvQkFBb0IscUJBQXFCO0FBQ3pDOztBQUVBO0FBQ0E7QUFDQTtBQUNBOztBQUVBO0FBQ0E7QUFDQTtBQUNBLFNBQVM7QUFDVDtBQUNBO0FBQ0E7O0FBRUE7QUFDQTtBQUNBOztBQUVBO0FBQ0EsRTs7Ozs7Ozs7OztBQ2pFYTs7QUFFYixpQ0FBaUMsMkhBQTJIOztBQUU1Siw2QkFBNkIsa0tBQWtLOztBQUUvTCxpREFBaUQsZ0JBQWdCLGdFQUFnRSx3REFBd0QsNkRBQTZELHNEQUFzRCxrSEFBa0g7O0FBRTlaLHNDQUFzQyx1REFBdUQsdUNBQXVDLFNBQVMsT0FBTyxrQkFBa0IsRUFBRSxhQUFhOztBQUVyTCx3Q0FBd0MsOEZBQThGLHdCQUF3QixlQUFlLGVBQWUsZ0JBQWdCLFlBQVksTUFBTSx3QkFBd0IsK0JBQStCLGFBQWEscUJBQXFCLG1DQUFtQyxFQUFFLEVBQUUsY0FBYyxXQUFXLFVBQVUsRUFBRSxVQUFVLE1BQU0saURBQWlELEVBQUUsVUFBVSxrQkFBa0IsRUFBRSxFQUFFLGFBQWE7O0FBRW5mLCtCQUErQixvQ0FBb0M7O0FBRW5FO0FBQ0E7QUFDQTtBQUNBOztBQUVBO0FBQ0E7QUFDQTs7QUFFQTtBQUNBO0FBQ0E7QUFDQSx1REFBdUQsY0FBYztBQUNyRTtBQUNBO0FBQ0E7QUFDQSxLQUFLO0FBQ0w7QUFDQTs7QUFFQTtBQUNBLEU7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7QUNsQ0EsTUFBcUc7QUFDckcsTUFBMkY7QUFDM0YsTUFBa0c7QUFDbEcsTUFBcUg7QUFDckgsTUFBOEc7QUFDOUcsTUFBOEc7QUFDOUcsTUFBeUc7Ozs7QUFJekc7O0FBRUEsNEJBQTRCLHFHQUFtQjtBQUMvQyx3QkFBd0Isa0hBQWE7O0FBRXJDLHVCQUF1Qix1R0FBYTs7QUFFcEMsaUJBQWlCLCtGQUFNO0FBQ3ZCLDZCQUE2QixzR0FBa0I7O0FBRS9DLGFBQWEsMEdBQUcsQ0FBQyxtRkFBTzs7OztBQUltRDtBQUMzRSxPQUFPLGlFQUFlLG1GQUFPLElBQUksMEZBQWMsR0FBRywwRkFBYyxZQUFZLEVBQUM7Ozs7Ozs7Ozs7O0FDMUJoRTs7QUFFYjs7QUFFQTtBQUNBOztBQUVBLGlCQUFpQix3QkFBd0I7QUFDekM7QUFDQTtBQUNBO0FBQ0E7QUFDQTs7QUFFQTtBQUNBOztBQUVBO0FBQ0E7QUFDQTs7QUFFQSxpQkFBaUIsaUJBQWlCO0FBQ2xDO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7O0FBRUE7QUFDQTtBQUNBO0FBQ0EsS0FBSztBQUNMO0FBQ0E7QUFDQTtBQUNBO0FBQ0EsT0FBTztBQUNQOztBQUVBO0FBQ0E7O0FBRUE7QUFDQTs7QUFFQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBOztBQUVBO0FBQ0EsS0FBSztBQUNMO0FBQ0E7QUFDQTtBQUNBOztBQUVBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTs7QUFFQSxtQkFBbUIsNEJBQTRCO0FBQy9DO0FBQ0E7QUFDQTtBQUNBOztBQUVBOztBQUVBLG9CQUFvQiw2QkFBNkI7QUFDakQ7O0FBRUE7O0FBRUE7QUFDQTs7QUFFQTtBQUNBO0FBQ0E7O0FBRUE7QUFDQTtBQUNBLEU7Ozs7Ozs7Ozs7QUNoR2E7O0FBRWI7QUFDQTs7QUFFQTtBQUNBO0FBQ0EscURBQXFEOztBQUVyRDtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0EsT0FBTztBQUNQO0FBQ0E7QUFDQTtBQUNBOztBQUVBO0FBQ0E7O0FBRUE7QUFDQTtBQUNBOzs7QUFHQTtBQUNBOztBQUVBO0FBQ0E7QUFDQTs7QUFFQTtBQUNBOztBQUVBLGtDOzs7Ozs7Ozs7O0FDdENhOztBQUViO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBOztBQUVBLG9DOzs7Ozs7Ozs7O0FDVmE7O0FBRWI7QUFDQTtBQUNBLGNBQWMsS0FBd0MsR0FBRyxzQkFBaUIsR0FBRyxDQUFJOztBQUVqRjtBQUNBO0FBQ0E7QUFDQTs7QUFFQSxnRDs7Ozs7Ozs7OztBQ1hhOztBQUViO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7O0FBRUE7QUFDQTtBQUNBLEdBQUc7QUFDSDtBQUNBOztBQUVBO0FBQ0EseURBQXlEO0FBQ3pELEdBQUc7O0FBRUg7OztBQUdBO0FBQ0E7O0FBRUE7QUFDQTtBQUNBO0FBQ0E7QUFDQTs7QUFFQTtBQUNBO0FBQ0E7OztBQUdBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQSxLQUFLO0FBQ0w7QUFDQTtBQUNBO0FBQ0E7QUFDQTs7QUFFQSx3Qjs7Ozs7Ozs7OztBQy9DYTs7QUFFYjtBQUNBO0FBQ0E7QUFDQTtBQUNBLEdBQUc7QUFDSDtBQUNBO0FBQ0E7O0FBRUE7QUFDQTtBQUNBOztBQUVBLG1DOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7OztBQ2RrQztBQUVsQyxvR0FBb0c7QUFDN0YsTUFBTSxVQUFVLEdBQVksR0FBRyxFQUN6QixhQUFhLEdBQVMsQ0FBQyxFQUN2QixNQUFNLEdBQWdCLE9BQU8sRUFDN0IsVUFBVSxHQUFZLElBQUksQ0FBQyxLQUFLLENBQUMsTUFBTSxDQUFDLEVBQ3hDLFlBQVksR0FBVSxJQUFJLEdBQUcsTUFBTSxFQUNuQyxhQUFhLEdBQVMsS0FBSyxHQUFDLFVBQVUsRUFDdEMsUUFBUSxHQUFjLGFBQWEsRUFDbkMsU0FBUyxHQUFhLGFBQWEsRUFDbkMsbUJBQW1CLEdBQUcsUUFBUSxFQUM5QixTQUFTLEdBQWEsRUFBRSxFQUN4QixXQUFXLEdBQVcsS0FBSztBQUVqQyxNQUFNLE1BQU0sR0FBRztJQUNwQixPQUFPLEVBQUssQ0FBQyxDQUFRLEVBQUUsRUFBRSxDQUFDLE1BQU0sQ0FBQyxDQUFDLENBQUM7SUFDbkMsT0FBTyxFQUFLLENBQUMsQ0FBUSxFQUFFLEVBQUUsQ0FBQyxDQUFDLENBQUMsR0FBQyxNQUFNLENBQUMsQ0FBQyxPQUFPLENBQUMsVUFBVSxDQUFDO0lBQ3hELFVBQVUsRUFBRSxDQUFDLENBQVEsRUFBRSxFQUFFLENBQUMsR0FBRyxNQUFNLENBQUMsT0FBTyxDQUFDLENBQUMsQ0FBQyxHQUFHO0NBQ2xEO0FBRUQsb0dBQW9HO0FBQzdGLE1BQU0sQ0FBQyxHQUFHLEVBQUUsQ0FBQyxFQUFFLENBQUMsRUFBRTtBQUV6QixNQUFNLEdBQUc7SUFBVDtRQUNFLGFBQVEsR0FBSSxhQUFhO1FBQ3pCLFlBQU8sR0FBSyxZQUFZO1FBQ3hCLGNBQVMsR0FBRyxhQUFhO0lBWTNCLENBQUM7SUFYQyxJQUFJO1FBQ0YsSUFBSSxDQUFDLENBQUMsQ0FBQyxHQUFHLElBQUksQ0FBQyxRQUFRLElBQUksQ0FBQyxFQUFFO1lBQzVCLE9BQU8sQ0FBQyxJQUFJLENBQUMsTUFBTSxFQUFFLElBQUksQ0FBQyxPQUFPLEVBQUUsSUFBSSxDQUFDLFNBQVMsQ0FBQztZQUNsRCxJQUFJLElBQUksQ0FBQyxTQUFTLEdBQUcsQ0FBQyxFQUFFO2dCQUN0QixJQUFJLENBQUMsT0FBTztnQkFDWixJQUFJLENBQUMsU0FBUyxJQUFJLENBQUM7Z0JBQ25CLE9BQU8sSUFBSSxDQUFDLE9BQU87YUFDcEI7U0FDRjtRQUNELE9BQU8sQ0FBQztJQUNWLENBQUM7Q0FDRjtBQUVNLE1BQU0sSUFBSTtJQWFmLFlBQWEsRUFBYTtRQVoxQixRQUFHLEdBQUcsSUFBSSxHQUFHLEVBQUU7UUFHZixnQkFBVyxHQUFXLENBQUM7UUFDdkIsYUFBUSxHQUFjLENBQUM7UUFDdkIsV0FBTSxHQUFnQixDQUFDO1FBQ3ZCLFlBQU8sR0FBZSxJQUFJLENBQUMsR0FBRyxDQUFDLElBQUksRUFBRTtRQUNyQyxZQUFPLEdBQWUsQ0FBQztRQUN2QixhQUFRLEdBQWMsQ0FBQztRQUN2QixjQUFTLEdBQWEsQ0FBQztRQUN2QixXQUFNLEdBQWdCLENBQUM7UUFHckIsSUFBSSxDQUFDLEVBQUUsR0FBRyxFQUFFO0lBQ2QsQ0FBQztJQUNELE1BQU07UUFDSixJQUFJLENBQUMsT0FBTyxJQUFJLElBQUksQ0FBQyxHQUFHLENBQUMsSUFBSSxFQUFFO1FBQy9CLElBQUksQ0FBQyxFQUFFLENBQUMsR0FBRyxDQUFDLEdBQUcsQ0FBQyxRQUFRLENBQUMsQ0FBQyxDQUFDLENBQUMsQ0FBQztRQUU3QixJQUFJLENBQUMsRUFBRSxDQUFDLEdBQUcsQ0FBQyxRQUFRLENBQUMsUUFBUSxDQUFDLElBQUksQ0FBQyxRQUFRLENBQUM7UUFDNUMsSUFBSSxDQUFDLEVBQUUsQ0FBQyxHQUFHLENBQUMsTUFBTSxDQUFDLFFBQVEsQ0FBQyxJQUFJLENBQUMsTUFBTSxDQUFDO1FBRXhDLElBQUksQ0FBQyxFQUFFLENBQUMsR0FBRyxDQUFDLE9BQU8sQ0FBQyxRQUFRLENBQUMsTUFBTSxDQUFDLE9BQU8sQ0FBQyxJQUFJLENBQUMsT0FBTyxDQUFDLENBQUM7UUFDMUQsSUFBSSxDQUFDLEVBQUUsQ0FBQyxHQUFHLENBQUMsT0FBTyxDQUFDLFFBQVEsQ0FBQyxNQUFNLENBQUMsT0FBTyxDQUFDLElBQUksQ0FBQyxPQUFPLENBQUMsQ0FBQztRQUMxRCxJQUFJLENBQUMsRUFBRSxDQUFDLEdBQUcsQ0FBQyxTQUFTLENBQUMsUUFBUSxDQUFDLElBQUksQ0FBQyxHQUFHLENBQUMsU0FBUyxDQUFDO1FBRWxELElBQUksQ0FBQyxFQUFFLENBQUMsR0FBRyxDQUFDLFFBQVEsQ0FBQyxRQUFRLENBQUMsSUFBSSxDQUFDLFFBQVEsQ0FBQztRQUM1QyxJQUFJLENBQUMsRUFBRSxDQUFDLEdBQUcsQ0FBQyxTQUFTLENBQUMsUUFBUSxDQUFDLElBQUksQ0FBQyxTQUFTLENBQUM7UUFDOUMsSUFBSSxDQUFDLEVBQUUsQ0FBQyxHQUFHLENBQUMsTUFBTSxDQUFDLFFBQVEsQ0FBQyxNQUFNLENBQUMsVUFBVSxDQUFDLElBQUksQ0FBQyxNQUFNLENBQUMsQ0FBQztJQUM3RCxDQUFDO0NBQ0Y7QUFFTSxNQUFNLElBQUk7SUFnQmYsWUFBYSxFQUFhLEVBQUUsSUFBVSxFQUFFLElBQVksRUFBRSxPQUFlO1FBWHJFLGdCQUFXLEdBQVksQ0FBQztRQUN4QixhQUFRLEdBQWUsQ0FBQztRQUN4QixXQUFNLEdBQWlCLENBQUM7UUFDeEIsUUFBRyxHQUFvQixDQUFDO1FBQ3hCLFdBQU0sR0FBaUIsQ0FBQztRQUN4QixZQUFPLEdBQWdCLENBQUM7UUFDeEIsY0FBUyxHQUFjLENBQUM7UUFDeEIsYUFBUSxHQUFlLENBQUM7UUFDeEIsV0FBTSxHQUFpQixDQUFDO1FBQ3hCLGlCQUFZLEdBQVcsQ0FBQztRQUN4QixVQUFLLEdBQWtCLENBQUM7UUFFdEIsSUFBSSxDQUFDLEVBQUUsR0FBUSxFQUFFO1FBQ2pCLElBQUksQ0FBQyxJQUFJLEdBQU0sSUFBSTtRQUNuQixJQUFJLENBQUMsSUFBSSxHQUFNLElBQUk7UUFDbkIsSUFBSSxDQUFDLE9BQU8sR0FBRyxPQUFPO0lBQ3hCLENBQUM7SUFDRCxNQUFNO1FBQ0osSUFBSSxDQUFDLEVBQUUsQ0FBQyxLQUFLLENBQUMsTUFBTSxDQUFDLElBQUksQ0FBQztJQUM1QixDQUFDO0lBQ0QsSUFBSSxDQUFFLE1BQWM7UUFDbEIsSUFBSSxDQUFDLEVBQUUsQ0FBQyxHQUFHLENBQUMsR0FBRyxDQUFDLE9BQU8sRUFBRSxJQUFJLENBQUMsSUFBSSxFQUFFLE1BQU0sQ0FBQztRQUMzQyxJQUFJLENBQUMsRUFBRSxDQUFDLE9BQU8sQ0FBQyxHQUFHLENBQUMsSUFBSSxDQUFDO1FBQ3pCLElBQUksQ0FBQyxFQUFFLENBQUMsT0FBTyxDQUFDLEdBQUcsQ0FBQyxJQUFJLENBQUM7SUFDM0IsQ0FBQztJQUNELFFBQVEsQ0FBRSxNQUFjO1FBQ3RCLElBQUksQ0FBQyxFQUFFLENBQUMsR0FBRyxDQUFDLEdBQUcsQ0FBQyxXQUFXLEVBQUUsSUFBSSxDQUFDLElBQUksRUFBRSxNQUFNLENBQUM7UUFDL0MsSUFBSSxJQUFJLENBQUMsTUFBTSxLQUFLLENBQUM7WUFBRSxJQUFJLENBQUMsRUFBRSxDQUFDLE9BQU8sQ0FBQyxNQUFNLENBQUMsSUFBSSxDQUFDO0lBQ3JELENBQUM7SUFDRCxLQUFLO1FBQ0gsTUFBTSxJQUFJLEtBQUssQ0FBQyxpQkFBaUIsQ0FBQztJQUNwQyxDQUFDO0lBQ0QsT0FBTyxDQUFFLE1BQWM7UUFDckIsT0FBTyxDQUFDLEtBQUssQ0FBQyxJQUFJLENBQUMsSUFBSSxFQUFFLE9BQU8sRUFBRSxNQUFNLENBQUM7UUFDekMsSUFBSSxNQUFNLElBQUksQ0FBQztZQUFFLE9BQU8sQ0FBQztRQUV6QixJQUFJLElBQUksQ0FBQyxNQUFNLEtBQUssQ0FBQztZQUFFLE9BQU8sQ0FBQztRQUUvQixJQUFJLElBQUksQ0FBQyxRQUFRLEdBQUcsQ0FBQyxJQUFJLElBQUksQ0FBQyxHQUFHLEdBQUcsU0FBUztZQUFFLE9BQU8sQ0FBQztRQUV2RCxJQUFJLElBQUksQ0FBQyxPQUFPLEdBQUcsSUFBSSxDQUFDLE1BQU0sRUFBRTtZQUM5QixJQUFJLENBQUMsRUFBRSxDQUFDLEdBQUcsQ0FBQyxHQUFHLENBQUMsZUFBZSxFQUFFLElBQUksQ0FBQyxJQUFJLEVBQUUsU0FBUyxDQUFDO1lBQ3RELE9BQU8sQ0FBQztTQUNUO1FBRUQsSUFBSSxNQUFNLEdBQUcsSUFBSSxDQUFDLElBQUksQ0FBQyxPQUFPLEVBQUU7WUFDOUIsSUFBSSxDQUFDLEVBQUUsQ0FBQyxHQUFHLENBQUMsR0FBRyxDQUFDLGVBQWUsRUFBRSxJQUFJLENBQUMsSUFBSSxFQUFFLFNBQVMsQ0FBQztZQUN0RCxPQUFPLENBQUM7U0FDVDtRQUVELElBQUksQ0FBQyxJQUFJLENBQUMsT0FBTyxJQUFJLE1BQU07UUFDM0IsSUFBSSxDQUFDLEVBQUUsQ0FBQyxHQUFHLENBQUMsR0FBRyxDQUFDLE9BQU8sRUFBRSxJQUFJLENBQUMsSUFBSSxFQUFFLE1BQU0sQ0FBQztRQUMzQyxPQUFPLENBQUMsS0FBSyxDQUFDLFVBQVUsRUFBRSxNQUFNLENBQUM7UUFDakMsT0FBTyxNQUFNO0lBQ2YsQ0FBQztJQUVELE1BQU07UUFDSixPQUFPLGdEQUFNLENBQUMsSUFBSSxDQUFDLElBQUksRUFBRSxJQUFJLENBQUM7SUFDaEMsQ0FBQztDQUNGOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7OztBQzNJeUM7QUFFMEI7QUFDTjtBQVk5RCxNQUFNLE9BQU87SUFBYjtRQUNFLFVBQUssR0FBRyxDQUFDO1FBQ1QsYUFBUSxHQUFHLElBQUksNERBQWMsRUFBRTtRQUMvQixVQUFLLEdBQUcsS0FBSztJQXNDZixDQUFDO0lBckNDLElBQUksQ0FBRSxHQUFXO1FBQ2YsSUFBSSxDQUFDLEtBQUssSUFBSSxDQUFDO1FBQ2YsSUFBSSxDQUFDLEtBQUssR0FBRywrQ0FBRztRQUNoQiwyREFBMkQ7UUFDM0QsTUFBTSxHQUFHLEdBQUcsZ0RBQU0sQ0FBQyxJQUFJLENBQUMsUUFBUSxDQUFDLElBQUksQ0FBQyxnREFBTSxDQUFDLEdBQUcsQ0FBQyxDQUFDLENBQUM7UUFDbkQsMkRBQTJEO1FBQzNELE9BQU8sR0FBRztJQUNaLENBQUM7SUFDRCxLQUFLLENBQUUsR0FBVztRQUNoQixJQUFJLENBQUMsS0FBSyxJQUFJLENBQUM7UUFDZixJQUFJLENBQUMsS0FBSyxHQUFHLCtDQUFHO1FBQ2hCLDREQUE0RDtRQUM1RCxNQUFNLEdBQUcsR0FBRyxnREFBTSxDQUFDLElBQUksQ0FBQyxRQUFRLENBQUMsS0FBSyxDQUFDLGdEQUFNLENBQUMsR0FBRyxDQUFDLENBQUMsQ0FBQztRQUNwRCw0REFBNEQ7UUFDNUQsT0FBTyxHQUFHO0lBQ1osQ0FBQztJQUNELE1BQU0sQ0FBRSxHQUFXO1FBQ2pCLElBQUksQ0FBQyxLQUFLLElBQUksQ0FBQztRQUNmLElBQUksQ0FBQyxLQUFLLEdBQUcsK0NBQUc7UUFDaEIsNkRBQTZEO1FBQzdELE1BQU0sR0FBRyxHQUFtQixnREFBTSxDQUFDLElBQUksQ0FBQyxRQUFRLENBQUMsTUFBTSxDQUFDLGdEQUFNLENBQUMsR0FBRyxDQUFDLENBQUMsQ0FBQztRQUNyRSxHQUFHLENBQUMsR0FBRyxHQUFHLE1BQU0sQ0FBQyxXQUFXLENBQUMsTUFBTTthQUNoQyxNQUFNLENBQUMsR0FBRyxDQUFDLEdBQWEsQ0FBQzthQUN6QixHQUFHLENBQUMsQ0FBQyxFQUFDLEdBQUcsRUFBRSxLQUFLLEVBQUMsRUFBQyxFQUFFLEVBQUMsR0FBRyxFQUFFLEtBQUssQ0FBQyxDQUFDLENBQUM7UUFDckMsSUFBSSxNQUFNLENBQUMsSUFBSSxDQUFDLEdBQUcsQ0FBQyxHQUFHLENBQUMsQ0FBQyxNQUFNLEdBQUcsQ0FBQztZQUFFLE9BQU8sQ0FBQyxHQUFHLENBQUMsR0FBRyxDQUFDLEdBQUcsQ0FBQztRQUN6RCw2REFBNkQ7UUFDN0QsT0FBTyxHQUFHO0lBQ1osQ0FBQztJQUNELElBQUksbUJBQW1CLENBQUUsUUFBZ0I7UUFDdkMsSUFBSSxDQUFDLFFBQVEsQ0FBQyxtQkFBbUIsR0FBRyxnREFBTSxDQUFDLFFBQVEsQ0FBQztJQUN0RCxDQUFDO0lBQ0QsSUFBSSxNQUFNLENBQUUsT0FBZTtRQUN6QixJQUFJLENBQUMsUUFBUSxDQUFDLE1BQU0sR0FBRyxnREFBTSxDQUFDLE9BQU8sQ0FBQztJQUN4QyxDQUFDO0lBQ0QsSUFBSSxLQUFLLENBQUUsTUFBYztRQUN2QixJQUFJLENBQUMsUUFBUSxDQUFDLEtBQUssR0FBRyxNQUFNLENBQUMsTUFBTSxDQUFDO0lBQ3RDLENBQUM7Q0FDRjtBQUVELG9HQUFvRztBQUNyRixTQUFlLFFBQVE7O1FBQ3BDLHlEQUF5RDtRQUN6RCw0REFBNEQ7UUFDNUQsTUFBTSxHQUFHLEdBQUcsSUFBSSxHQUFHLENBQUMsaUJBQWlCLEVBQUUsUUFBUSxDQUFDLElBQUksQ0FBQyxFQUMvQyxHQUFHLEdBQUcsTUFBTSxLQUFLLENBQUMsR0FBRyxDQUFDLFFBQVEsRUFBRSxDQUFDLEVBQ2pDLEdBQUcsR0FBRyxNQUFNLEdBQUcsQ0FBQyxXQUFXLEVBQUU7UUFDbkMsTUFBTSwrREFBVyxDQUFDLEdBQUcsQ0FBQztJQUN4QixDQUFDO0NBQUE7QUFFRCxvR0FBb0c7QUFDN0YsTUFBTSxRQUFTLFNBQVEsZ0RBQUk7SUFFaEMsWUFBYSxFQUFhO1FBQ3hCLEtBQUssQ0FBQyxFQUFFLENBQUM7UUFGWCxhQUFRLEdBQVksSUFBSSxPQUFPLEVBQUU7UUFHL0IsSUFBSSxDQUFDLFFBQVEsQ0FBQyxJQUFJLENBQUM7WUFDakIsWUFBWSxFQUFFLEVBQUUsT0FBTyxFQUFFLEVBQUUsRUFBRSxTQUFTLEVBQUUsRUFBRSxFQUFFO1lBQzVDLFFBQVEsRUFBTSxFQUFFLE9BQU8sRUFBRSxFQUFFLEVBQUUsU0FBUyxFQUFFLEVBQUUsRUFBRTtZQUM1QyxXQUFXLEVBQUcsRUFBRTtZQUNoQixTQUFTLEVBQUsscURBQVM7WUFDdkIsUUFBUSxFQUFNLG9EQUFRO1NBQ3ZCLENBQUM7UUFDRixJQUFJLENBQUMsRUFBRSxDQUFDLEdBQUcsQ0FBQyxLQUFLLENBQUMsT0FBTyxHQUFHLElBQUksQ0FBQyxLQUFLLENBQUMsSUFBSSxDQUFDLElBQUksQ0FBQztJQUNuRCxDQUFDO0lBQ0QsTUFBTTtRQUNKLElBQUksQ0FBQyxRQUFRLENBQUMsbUJBQW1CLEdBQUcsRUFBQyxPQUFPLEVBQUMsRUFBQyxNQUFNLEVBQUMsTUFBTSxDQUFDLElBQUksQ0FBQyxPQUFPLENBQUMsRUFBQyxFQUFDO1FBQzNFLE1BQU0sSUFBSSxHQUFHLElBQUksQ0FBQyxRQUFRLENBQUMsS0FBSyxDQUFDLEVBQUMsU0FBUyxFQUFDLEVBQUMsRUFBRSxFQUFDLCtDQUFHLEVBQUMsRUFBQyxDQUFDLENBQUMsU0FBUztRQUNoRSxtQkFBbUI7UUFDbkIsSUFBSSxDQUFDLFdBQVcsR0FBRyxJQUFJLENBQUMsZ0JBQWdCO1FBQ3hDLElBQUksQ0FBQyxRQUFRLEdBQU0sSUFBSSxDQUFDLGFBQWE7UUFDckMsSUFBSSxDQUFDLE1BQU0sR0FBUSxJQUFJLENBQUMsV0FBVztRQUNuQyxJQUFJLENBQUMsT0FBTyxHQUFPLElBQUksQ0FBQyxZQUFZO1FBQ3BDLElBQUksQ0FBQyxTQUFTLEdBQUssSUFBSSxDQUFDLGNBQWM7UUFDdEMsSUFBSSxDQUFDLFFBQVEsR0FBTSxJQUFJLENBQUMsYUFBYTtRQUNyQyxJQUFJLENBQUMsTUFBTSxHQUFRLElBQUksQ0FBQyxXQUFXO1FBQ25DLEtBQUssQ0FBQyxNQUFNLEVBQUU7SUFDaEIsQ0FBQztJQUNELEtBQUs7UUFDSCxJQUFJLENBQUMsUUFBUSxDQUFDLE1BQU0sR0FBRyxFQUFFO1FBQ3pCLElBQUksQ0FBQyxRQUFRLENBQUMsTUFBTSxDQUFDLEVBQUMsVUFBVSxFQUFDLEVBQUMsT0FBTyxFQUFDLGFBQWEsRUFBQyxFQUFDLENBQUM7SUFDNUQsQ0FBQztDQUNGO0FBRUQsb0dBQW9HO0FBQzdGLE1BQU0sUUFBUyxTQUFRLGdEQUFJO0lBUWhDLFlBQWEsRUFBYSxFQUFFLElBQVUsRUFBRSxJQUFZLEVBQUUsT0FBZTtRQUNuRSxLQUFLLENBQUMsRUFBRSxFQUFFLElBQUksRUFBRSxJQUFJLEVBQUUsT0FBTyxDQUFDO1FBQzlCLElBQUksQ0FBQyxPQUFPLEdBQUcsSUFBSSxDQUFDLElBQUk7UUFDeEIsSUFBSSxDQUFDLFFBQVEsQ0FBQyxNQUFNLEdBQUcsSUFBSSxDQUFDLE9BQU87UUFDbkMsSUFBSSxDQUFDLFFBQVEsQ0FBQyxNQUFNLENBQUMsRUFBRSxlQUFlLEVBQUUsRUFBRSxHQUFHLEVBQUUsRUFBRSxFQUFFLEVBQUUsQ0FBQztJQUN4RCxDQUFDO0lBVEQsSUFBSSxRQUFRO1FBQ1YsT0FBUSxJQUFJLENBQUMsSUFBaUIsQ0FBQyxRQUFRO0lBQ3pDLENBQUM7SUFTRCxNQUFNO1FBQ0osZ0VBQWdFO1FBQ2hFLHlEQUF5RDtRQUN6RCxJQUFJLENBQUMsUUFBUSxDQUFDLG1CQUFtQixHQUFHLEVBQUMsT0FBTyxFQUFDLEVBQUMsTUFBTSxFQUFDLE1BQU0sQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDLE9BQU8sQ0FBQyxFQUFDLEVBQUM7UUFFaEYsdUVBQXVFO1FBQ3ZFLG1DQUFtQztRQUNuQyxNQUFNLElBQUksR0FBRyxJQUFJLENBQUMsUUFBUSxDQUFDLEtBQUssQ0FBQyxFQUFDLFNBQVMsRUFBRSxFQUFFLEVBQUUsRUFBRSwrQ0FBRyxFQUFFLE9BQU8sRUFBRSxJQUFJLENBQUMsT0FBTyxFQUFFLEdBQUcsRUFBRSxFQUFFLEVBQUUsRUFBQyxDQUFDLENBQUMsU0FBUztRQUNwRyxJQUFJLENBQUMsV0FBVyxHQUFHLElBQUksQ0FBQyxnQkFBZ0I7UUFDeEMsSUFBSSxDQUFDLFFBQVEsR0FBTSxNQUFNLENBQUMsSUFBSSxDQUFDLGFBQWEsQ0FBQztRQUM3QyxJQUFJLENBQUMsTUFBTSxHQUFRLE1BQU0sQ0FBQyxJQUFJLENBQUMsV0FBVyxDQUFDO1FBQzNDLElBQUksQ0FBQyxLQUFLLEdBQVMsTUFBTSxDQUFDLElBQUksQ0FBQyxVQUFVLENBQUM7UUFDMUMsSUFBSSxDQUFDLEdBQUcsR0FBVyxNQUFNLENBQUMsSUFBSSxDQUFDLFFBQVEsQ0FBQztRQUN4QyxJQUFJLENBQUMsTUFBTSxHQUFRLE1BQU0sQ0FBQyxJQUFJLENBQUMsV0FBVyxDQUFDO1FBQzNDLElBQUksQ0FBQyxPQUFPLEdBQU8sTUFBTSxDQUFDLElBQUksQ0FBQyxZQUFZLENBQUM7UUFDNUMsSUFBSSxDQUFDLFNBQVMsR0FBSyxNQUFNLENBQUMsSUFBSSxDQUFDLGNBQWMsQ0FBQztRQUM5QyxJQUFJLENBQUMsUUFBUSxHQUFNLE1BQU0sQ0FBQyxJQUFJLENBQUMsYUFBYSxDQUFDO1FBQzdDLEtBQUssQ0FBQyxNQUFNLEVBQUU7SUFDaEIsQ0FBQztJQUVELElBQUksQ0FBRSxNQUFjO1FBQ2xCLElBQUksQ0FBQyxRQUFRLENBQUMsTUFBTSxHQUFHLElBQUksQ0FBQyxPQUFPO1FBQ25DLElBQUk7WUFDRiwrQkFBK0I7WUFDL0IsSUFBSSxDQUFDLFFBQVEsQ0FBQyxNQUFNLENBQUMsRUFBRSxJQUFJLEVBQUUsRUFBRSxNQUFNLEVBQUUsTUFBTSxDQUFDLE1BQU0sQ0FBQyxFQUFFLEVBQUUsQ0FBQztZQUMxRCxLQUFLLENBQUMsSUFBSSxDQUFDLE1BQU0sQ0FBQztTQUNuQjtRQUFDLE9BQU8sQ0FBQyxFQUFFO1lBQ1Ysa0JBQWtCO1NBQ25CO0lBQ0gsQ0FBQztJQUVELFFBQVEsQ0FBRSxNQUFjO1FBQ3RCLElBQUksQ0FBQyxRQUFRLENBQUMsTUFBTSxHQUFHLElBQUksQ0FBQyxPQUFPO1FBQ25DLElBQUk7WUFDRixtQ0FBbUM7WUFDbkMsSUFBSSxDQUFDLFFBQVEsQ0FBQyxNQUFNLENBQUMsRUFBRSxRQUFRLEVBQUUsRUFBRSxNQUFNLEVBQUUsTUFBTSxDQUFDLE1BQU0sQ0FBQyxFQUFFLEVBQUUsQ0FBQztZQUM5RCxLQUFLLENBQUMsUUFBUSxDQUFDLE1BQU0sQ0FBQztTQUN2QjtRQUFDLE9BQU8sQ0FBQyxFQUFFO1lBQ1Ysa0JBQWtCO1NBQ25CO0lBQ0gsQ0FBQztJQUVELEtBQUs7UUFDSCxJQUFJLENBQUMsUUFBUSxDQUFDLE1BQU0sR0FBRyxJQUFJLENBQUMsT0FBTztRQUNuQyxJQUFJO1lBQ0YsTUFBTSxNQUFNLEdBQUcsSUFBSSxDQUFDLFFBQVEsQ0FBQyxNQUFNLENBQUMsRUFBRSxLQUFLLEVBQUUsRUFBRSxFQUFFLENBQUM7WUFDbEQsTUFBTSxNQUFNLEdBQUcsTUFBTSxDQUFDLE1BQU0sQ0FBQyxHQUFHLENBQUMsTUFBTSxDQUFDO1lBQ3hDLE9BQU8sSUFBSSxDQUFDLE9BQU8sQ0FBQyxNQUFNLENBQUM7U0FDNUI7UUFBQyxPQUFPLENBQUMsRUFBRTtZQUNWLE9BQU8sQ0FBQyxLQUFLLENBQUMsQ0FBQyxDQUFDO1lBQ2hCLE9BQU8sQ0FBQztTQUNUO0lBQ0gsQ0FBQztDQUNGOzs7Ozs7Ozs7Ozs7Ozs7OztBQzFLRCxtRUFBbUU7QUFFbkUsTUFBTSxTQUFTLEdBQUcsU0FBUztBQUMzQixNQUFNLEtBQUssR0FBRyxTQUFTO0FBQ3ZCLE1BQU0sU0FBUyxHQUFHLFNBQVM7QUFDM0IsTUFBTSxLQUFLLEdBQUcsU0FBUztBQUN2QixNQUFNLEtBQUssR0FBRyxTQUFTO0FBQ3ZCLE1BQU0sS0FBSyxHQUFHLFNBQVM7QUFDdkIsTUFBTSxLQUFLLEdBQUcsU0FBUztBQUV2QixNQUFNLE9BQU8sR0FBRyxTQUFTO0FBQ3pCLE1BQU0sT0FBTyxHQUFHLFNBQVM7QUFFekIsTUFBTSxVQUFVLEdBQUcsU0FBUztBQUM1QixNQUFNLE1BQU0sR0FBRyxTQUFTO0FBQ3hCLE1BQU0sVUFBVSxHQUFHLFNBQVM7QUFDNUIsTUFBTSxNQUFNLEdBQUcsU0FBUztBQUN4QixNQUFNLE1BQU0sR0FBRyxTQUFTO0FBQ3hCLE1BQU0sTUFBTSxHQUFHLFNBQVM7QUFDeEIsTUFBTSxNQUFNLEdBQUcsU0FBUztBQUV4QixNQUFNLFNBQVMsR0FBRyxTQUFTO0FBQzNCLE1BQU0sV0FBVyxHQUFHLFNBQVM7QUFDN0IsTUFBTSxZQUFZLEdBQUcsU0FBUztBQUM5QixNQUFNLFVBQVUsR0FBRyxTQUFTO0FBQzVCLE1BQU0sWUFBWSxHQUFHLFNBQVM7QUFDOUIsTUFBTSxVQUFVLEdBQUcsU0FBUztBQUM1QixNQUFNLFlBQVksR0FBRyxTQUFTO0FBRTlCLE1BQU0sVUFBVSxHQUFHLFNBQVM7QUFDNUIsTUFBTSxZQUFZLEdBQUcsU0FBUztBQUM5QixNQUFNLGFBQWEsR0FBRyxTQUFTO0FBQy9CLE1BQU0sV0FBVyxHQUFHLFNBQVM7QUFDN0IsTUFBTSxhQUFhLEdBQUcsU0FBUztBQUMvQixNQUFNLFdBQVcsR0FBRyxTQUFTO0FBQzdCLE1BQU0sYUFBYSxHQUFHLFNBQVM7QUFFL0IsTUFBTSxRQUFRLEdBQUcsU0FBUztBQUMxQixNQUFNLFVBQVUsR0FBRyxTQUFTO0FBQzVCLE1BQU0sV0FBVyxHQUFHLFNBQVM7QUFDN0IsTUFBTSxTQUFTLEdBQUcsU0FBUztBQUMzQixNQUFNLFdBQVcsR0FBRyxTQUFTO0FBQzdCLE1BQU0sU0FBUyxHQUFHLFNBQVM7QUFDM0IsTUFBTSxXQUFXLEdBQUcsU0FBUztBQUU3QixNQUFNLE9BQU8sR0FBRztJQUNkLFNBQVM7SUFDVCxTQUFTO0lBQ1QsS0FBSztJQUNMLEtBQUs7SUFDTCxLQUFLO0lBQ0wsS0FBSztJQUNMLEtBQUs7SUFDTCxJQUFJLEVBQUU7UUFDSixJQUFJLEVBQUUsU0FBUztRQUNmLElBQUksRUFBRSxTQUFTO1FBQ2YsQ0FBQyxFQUFFLEtBQUs7UUFDUixDQUFDLEVBQUUsS0FBSztRQUNSLENBQUMsRUFBRSxLQUFLO1FBQ1IsQ0FBQyxFQUFFLEtBQUs7UUFDUixDQUFDLEVBQUUsS0FBSztLQUNUO0lBRUQsT0FBTztJQUNQLE9BQU87SUFDUCxJQUFJLEVBQUU7UUFDSixHQUFHLEVBQUUsT0FBTztRQUNaLEdBQUcsRUFBRSxPQUFPO0tBQ2I7SUFFRCxVQUFVO0lBQ1YsVUFBVTtJQUNWLE1BQU07SUFDTixNQUFNO0lBQ04sTUFBTTtJQUNOLE1BQU07SUFDTixNQUFNO0lBQ04sS0FBSyxFQUFFO1FBQ0wsSUFBSSxFQUFFLFVBQVU7UUFDaEIsSUFBSSxFQUFFLFVBQVU7UUFDaEIsQ0FBQyxFQUFFLE1BQU07UUFDVCxDQUFDLEVBQUUsTUFBTTtRQUNULENBQUMsRUFBRSxNQUFNO1FBQ1QsQ0FBQyxFQUFFLE1BQU07UUFDVCxDQUFDLEVBQUUsTUFBTTtLQUNWO0lBRUQsU0FBUztJQUNULFdBQVc7SUFDWCxZQUFZO0lBQ1osVUFBVTtJQUNWLFlBQVk7SUFDWixVQUFVO0lBQ1YsWUFBWTtJQUNaLE1BQU0sRUFBRTtRQUNOLEdBQUcsRUFBRSxTQUFTO1FBQ2QsS0FBSyxFQUFFLFdBQVc7UUFDbEIsTUFBTSxFQUFFLFlBQVk7UUFDcEIsSUFBSSxFQUFFLFVBQVU7UUFDaEIsTUFBTSxFQUFFLFlBQVk7UUFDcEIsSUFBSSxFQUFFLFVBQVU7UUFDaEIsTUFBTSxFQUFFLFlBQVk7S0FDckI7SUFFRCxVQUFVO0lBQ1YsWUFBWTtJQUNaLGFBQWE7SUFDYixXQUFXO0lBQ1gsYUFBYTtJQUNiLFdBQVc7SUFDWCxhQUFhO0lBQ2IsT0FBTyxFQUFFO1FBQ1AsR0FBRyxFQUFFLFVBQVU7UUFDZixLQUFLLEVBQUUsWUFBWTtRQUNuQixNQUFNLEVBQUUsYUFBYTtRQUNyQixJQUFJLEVBQUUsV0FBVztRQUNqQixNQUFNLEVBQUUsYUFBYTtRQUNyQixJQUFJLEVBQUUsV0FBVztRQUNqQixNQUFNLEVBQUUsYUFBYTtLQUN0QjtJQUVELFFBQVE7SUFDUixVQUFVO0lBQ1YsV0FBVztJQUNYLFNBQVM7SUFDVCxXQUFXO0lBQ1gsU0FBUztJQUNULFdBQVc7SUFDWCxLQUFLLEVBQUU7UUFDTCxHQUFHLEVBQUUsUUFBUTtRQUNiLEtBQUssRUFBRSxVQUFVO1FBQ2pCLE1BQU0sRUFBRSxXQUFXO1FBQ25CLElBQUksRUFBRSxTQUFTO1FBQ2YsTUFBTSxFQUFFLFdBQVc7UUFDbkIsSUFBSSxFQUFFLFNBQVM7UUFDZixNQUFNLEVBQUUsV0FBVztLQUNwQjtDQUNGO0FBRUQsaUVBQWUsT0FBTztBQUVpQztBQUNoRCxNQUFNLE1BQU0sR0FBRyxNQUFNLENBQUMsTUFBTSxDQUNqQyxTQUFTLFFBQVEsQ0FBRSxJQUFVLEVBQUUsSUFBVTtJQUN2QyxRQUFRLElBQUksRUFBRTtRQUNaLEtBQUssSUFBSSxDQUFDLEdBQUcsR0FBRyxxREFBUyxJQUFJLElBQUksQ0FBQyxRQUFRLEdBQUcsQ0FBQyxFQUFFLDRCQUE0QjtZQUMxRSxPQUFPLE1BQU0sQ0FBQyxRQUFRO1FBQ3hCLEtBQUssSUFBSSxDQUFDLFNBQVMsR0FBRyxDQUFDLElBQUksSUFBSSxDQUFDLFFBQVEsSUFBSSxDQUFDLEVBQUcsd0JBQXdCO1lBQ3RFLE9BQU8sTUFBTSxDQUFDLFFBQVE7UUFDeEIsNkVBQTZFO1FBQzNFLHNCQUFzQjtRQUN4QixLQUFLLElBQUksQ0FBQyxTQUFTLEdBQUcsSUFBSSxDQUFDLE9BQU8sRUFBYywyQkFBMkI7WUFDekUsT0FBTyxNQUFNLENBQUMsT0FBTztRQUN2QixLQUFLLElBQUksQ0FBQyxPQUFPLEdBQUcsSUFBSSxDQUFDLE1BQU0sRUFBaUIsY0FBYztZQUM1RCxPQUFPLE1BQU0sQ0FBQyxPQUFPO1FBQ3ZCLEtBQUssSUFBSSxDQUFDLFNBQVMsS0FBSyxDQUFDO1lBQ3ZCLE9BQU8sTUFBTSxDQUFDLE9BQU87UUFDdkI7WUFDRSxPQUFPLE1BQU0sQ0FBQyxTQUFTO0tBQzFCO0FBQ0gsQ0FBQyxFQUFFO0lBQ0QsU0FBUyxFQUFFLENBQUMsT0FBTyxDQUFDLFNBQVMsRUFBSSxPQUFPLENBQUMsVUFBVSxDQUFDO0lBQ3BELFFBQVEsRUFBRyxDQUFDLE9BQU8sQ0FBQyxVQUFVLEVBQUcsT0FBTyxDQUFDLFVBQVUsQ0FBQztJQUNwRCxPQUFPLEVBQUksQ0FBQyxPQUFPLENBQUMsV0FBVyxFQUFFLE9BQU8sQ0FBQyxZQUFZLENBQUM7SUFDdEQsT0FBTyxFQUFJLENBQUMsT0FBTyxDQUFDLFdBQVcsRUFBRSxPQUFPLENBQUMsWUFBWSxDQUFDO0lBQ3RELFFBQVEsRUFBRyxDQUFDLE9BQU8sQ0FBQyxTQUFTLEVBQUksT0FBTyxDQUFDLFVBQVUsQ0FBQztJQUNwRCxPQUFPLEVBQUksQ0FBQyxPQUFPLENBQUMsS0FBSyxFQUFRLE9BQU8sQ0FBQyxZQUFZLENBQUM7Q0FDdkQsQ0FBQzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7QUN2S0osb0dBQW9HO0FBRTdGLE1BQU0sTUFBTSxHQUFHLENBQUMsR0FBVyxFQUFFLEVBQUUsQ0FDcEMsSUFBSSxDQUFDLEtBQUssQ0FBQyxJQUFJLENBQUMsTUFBTSxFQUFFLEdBQUMsR0FBRyxDQUFDO0FBRXhCLE1BQU0sVUFBVSxHQUFHLENBQUMsQ0FBTSxFQUFFLEVBQUUsQ0FDbkMsQ0FBQyxDQUFDLE1BQU0sQ0FBQyxDQUFDLENBQUMsTUFBTSxDQUFDLENBQUM7QUFFckIsb0dBQW9HO0FBRTdGLFNBQVMsUUFBUSxDQUFFLENBQVMsRUFBRSxFQUFZO0lBQy9DLCtFQUErRTtJQUMvRSxJQUFJLE9BQVk7SUFDaEIsT0FBTyxTQUFTLFNBQVMsQ0FBRSxHQUFHLElBQVE7UUFDcEMsT0FBTyxJQUFJLE9BQU8sQ0FBQyxPQUFPLEdBQUU7WUFDMUIsSUFBSSxPQUFPO2dCQUFFLFlBQVksQ0FBQyxPQUFPLENBQUM7WUFDbEMsT0FBTyxHQUFHLEtBQUssQ0FBQyxDQUFDLEVBQUUsR0FBRSxFQUFFLFFBQU8sQ0FBQyxFQUFFLENBQUMsR0FBRyxJQUFJLENBQUMsQ0FBQyxDQUFDO1FBQUMsQ0FBQyxDQUFDO0lBQUEsQ0FBQztBQUFBLENBQUM7QUFFaEQsU0FBUyxLQUFLLENBQUUsQ0FBUyxFQUFFLEVBQVk7SUFDNUMsT0FBTyxVQUFVLENBQUMsRUFBRSxFQUFFLENBQUMsQ0FBQztBQUFDLENBQUM7QUFFNUIsb0dBQW9HO0FBRTdGLFNBQVMsQ0FBQyxDQUFFLE9BQWUsRUFBRSxVQUFVLEdBQUMsRUFBRSxFQUFFLEdBQUcsT0FBVztJQUMvRCxNQUFNLEVBQUUsR0FBRyxNQUFNLENBQUMsTUFBTSxDQUFDLFFBQVEsQ0FBQyxhQUFhLENBQUMsT0FBTyxDQUFDLEVBQUUsVUFBVSxDQUFDO0lBQ3JFLEtBQUssTUFBTSxHQUFHLElBQUksT0FBTztRQUFFLEVBQUUsQ0FBQyxXQUFXLENBQUMsR0FBRyxDQUFDO0lBQzlDLE9BQU8sRUFBRTtBQUFDLENBQUM7QUFFTixTQUFTLE1BQU0sQ0FBRSxNQUFtQixFQUFFLEtBQWtCO0lBQzdELE9BQU8sTUFBTSxDQUFDLFdBQVcsQ0FBQyxLQUFLLENBQUM7QUFBQyxDQUFDO0FBRTdCLFNBQVMsT0FBTyxDQUFFLE1BQW1CLEVBQUUsS0FBa0I7SUFDOUQsT0FBTyxNQUFNLENBQUMsWUFBWSxDQUFDLEtBQUssRUFBRSxNQUFNLENBQUMsVUFBVSxDQUFDO0FBQUMsQ0FBQztBQUV4RCxvR0FBb0c7QUFFcEcsTUFBTSxHQUFHLEdBQUcsSUFBSSxXQUFXLEVBQUU7QUFDdEIsTUFBTSxNQUFNLEdBQUcsQ0FBQyxDQUFNLEVBQUUsRUFBRSxDQUFDLEdBQUcsQ0FBQyxNQUFNLENBQUMsSUFBSSxDQUFDLFNBQVMsQ0FBQyxDQUFDLENBQUMsQ0FBQztBQUUvRCxNQUFNLEdBQUcsR0FBRyxJQUFJLFdBQVcsRUFBRTtBQUN0QixNQUFNLE1BQU0sR0FBRyxDQUFDLENBQWEsRUFBRSxFQUFFLENBQUMsSUFBSSxDQUFDLEtBQUssQ0FBQyxHQUFHLENBQUMsTUFBTSxDQUFDLENBQUMsQ0FBQyxNQUFNLENBQUMsQ0FBQzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7QUN4QzNCO0FBQ1U7QUFFeEQsb0dBQW9HO0FBQzdGLE1BQU0sVUFBVSxHQUFHLElBQUk7QUFDdkIsTUFBTSxRQUFRLEdBQUssS0FBSztBQVcvQixnQkFBZ0I7QUFDVCxNQUFNLEtBQUs7SUFJaEIsWUFBYSxJQUFZLEVBQUUsS0FBVztRQUh0QyxTQUFJLEdBQUksMkNBQUMsQ0FBQyxLQUFLLEVBQUUsRUFBRSxTQUFTLEVBQUUsT0FBTyxFQUFFLENBQUM7UUFDeEMsVUFBSyxHQUFHLGdEQUFNLENBQUMsSUFBSSxDQUFDLElBQUksRUFBRSwyQ0FBQyxDQUFDLE9BQU8sQ0FBQyxDQUFDO1FBQ3JDLFVBQUssR0FBRyxnREFBTSxDQUFDLElBQUksQ0FBQyxJQUFJLEVBQUUsMkNBQUMsQ0FBQyxLQUFLLENBQUMsQ0FBQztRQUVqQyxJQUFJLENBQUMsS0FBSyxDQUFDLFdBQVcsR0FBRyxJQUFJO1FBQzdCLElBQUksQ0FBQyxLQUFLLENBQUMsV0FBVyxHQUFHLE1BQU0sQ0FBQyxLQUFLLENBQUM7SUFDeEMsQ0FBQztJQUNELE1BQU0sQ0FBRSxNQUFtQjtRQUN6QixNQUFNLENBQUMsV0FBVyxDQUFDLElBQUksQ0FBQyxJQUFJLENBQUM7UUFDN0IsT0FBTyxJQUFJO0lBQ2IsQ0FBQztJQUNELFFBQVEsQ0FBRSxLQUFVO1FBQ2xCLElBQUksQ0FBQyxLQUFLLENBQUMsV0FBVyxHQUFHLE1BQU0sQ0FBQyxLQUFLLENBQUM7SUFDeEMsQ0FBQztDQUNGO0FBRUQsb0dBQW9HO0FBQzdGLE1BQU0sR0FBRztJQUFoQjtRQUNFLFNBQUksR0FBUSwyQ0FBQyxDQUFDLEtBQUssRUFBRSxFQUFFLFNBQVMsRUFBRSxTQUFTLEVBQUUsQ0FBQztRQUM5QyxTQUFJLEdBQVEsZ0RBQU0sQ0FBQyxJQUFJLENBQUMsSUFBSSxFQUFFLDJDQUFDLENBQUMsSUFBSSxDQUFDLENBQUM7UUFFdEMsUUFBRyxHQUFTLElBQUksS0FBSyxDQUFDLE9BQU8sQ0FBQyxDQUFDLE1BQU0sQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDO1FBRWhELFdBQU0sR0FBTSxJQUFJLEtBQUssQ0FBQyx1QkFBdUIsQ0FBQyxDQUFDLE1BQU0sQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDO1FBQ2hFLGFBQVEsR0FBSSxJQUFJLEtBQUssQ0FBQyw0QkFBNEIsQ0FBQyxDQUFDLE1BQU0sQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDO1FBRXJFLFlBQU8sR0FBSyxJQUFJLEtBQUssQ0FBQywwQkFBMEIsQ0FBQyxDQUFDLE1BQU0sQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDO1FBQ25FLFlBQU8sR0FBSyxJQUFJLEtBQUssQ0FBQywwQkFBMEIsQ0FBQyxDQUFDLE1BQU0sQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDO1FBQ25FLGNBQVMsR0FBRyxJQUFJLEtBQUssQ0FBQyw0QkFBNEIsQ0FBQyxDQUFDLE1BQU0sQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDO1FBRXJFLGNBQVMsR0FBRyxJQUFJLEtBQUssQ0FBQyx1QkFBdUIsQ0FBQyxDQUFDLE1BQU0sQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDO1FBQ2hFLGFBQVEsR0FBSSxJQUFJLEtBQUssQ0FBQyxzQkFBc0IsQ0FBQyxDQUFDLE1BQU0sQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDO1FBQy9ELFdBQU0sR0FBTSxJQUFJLEtBQUssQ0FBQyxzQkFBc0IsQ0FBQyxDQUFDLE1BQU0sQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDO1FBRS9ELFVBQUssR0FBRyxnREFBTSxDQUFDLElBQUksQ0FBQyxJQUFJLEVBQUUsMkNBQUMsQ0FBQyxRQUFRLEVBQUUsRUFBRSxXQUFXLEVBQUUsWUFBWSxFQUFFLENBQUMsQ0FBQztJQVV2RSxDQUFDO0lBUkMsR0FBRyxDQUFFLEtBQWEsRUFBRSxJQUFZLEVBQUUsTUFBd0I7UUFDeEQsSUFBSSxVQUFVO1lBQUUsT0FBTTtRQUN0QixJQUFJLE1BQU0sRUFBRTtZQUNWLGlEQUFPLENBQUMsSUFBSSxDQUFDLElBQUksRUFBRSwyQ0FBQyxDQUFDLEtBQUssRUFBRSxFQUFFLFNBQVMsRUFBRSxNQUFNLElBQUksUUFBUSxLQUFLLElBQUksTUFBTSxJQUFJLEVBQUUsQ0FBQyxDQUFDO1NBQ25GO2FBQU07WUFDTCxpREFBTyxDQUFDLElBQUksQ0FBQyxJQUFJLEVBQUUsMkNBQUMsQ0FBQyxLQUFLLEVBQUUsRUFBRSxTQUFTLEVBQUUsTUFBTSxJQUFJLFFBQVEsS0FBSyxFQUFFLEVBQUUsQ0FBQyxDQUFDO1NBQ3ZFO0lBQ0gsQ0FBQztDQUNGO0FBcUJNLE1BQU0sS0FBSztJQUloQjtRQUZBLFNBQUksR0FBUyxFQUFFLENBQUM7UUFHZCxJQUFJLENBQUMsSUFBSSxHQUFHLFFBQVEsQ0FBQyxhQUFhLENBQUMsT0FBTyxDQUFDO1FBQzNDLElBQUksUUFBUTtZQUFFLE9BQU07SUFDdEIsQ0FBQztJQUVELElBQUksQ0FBRSxLQUFZO1FBQ2hCLGdEQUFNLENBQUMsSUFBSSxDQUFDLElBQUksRUFBRSwyQ0FBQyxDQUFDLE9BQU8sRUFBRSxFQUFFLEVBQzdCLDJDQUFDLENBQUMsSUFBSSxFQUFFLEVBQUUsV0FBVyxFQUFFLE1BQU0sRUFBVSxDQUFDLEVBQ3hDLDJDQUFDLENBQUMsSUFBSSxFQUFFLEVBQUUsV0FBVyxFQUFFLGFBQWEsRUFBRyxDQUFDLEVBQ3hDLDJDQUFDLENBQUMsSUFBSSxFQUFFLEVBQUUsV0FBVyxFQUFFLEtBQUssRUFBVyxDQUFDLEVBQ3hDLDJDQUFDLENBQUMsSUFBSSxFQUFFLEVBQUUsV0FBVyxFQUFFLFFBQVEsRUFBUSxDQUFDLEVBQ3hDLDJDQUFDLENBQUMsSUFBSSxFQUFFLEVBQUUsV0FBVyxFQUFFLFVBQVUsRUFBTSxDQUFDLEVBQ3hDLDJDQUFDLENBQUMsSUFBSSxFQUFFLEVBQUUsV0FBVyxFQUFFLE9BQU8sRUFBUyxDQUFDLEVBQ3hDLDJDQUFDLENBQUMsSUFBSSxFQUFFLEVBQUUsV0FBVyxFQUFFLFFBQVEsRUFBUSxDQUFDLEVBQ3hDLDJDQUFDLENBQUMsSUFBSSxFQUFFLEVBQUUsV0FBVyxFQUFFLFNBQVMsRUFBTyxDQUFDLEVBQ3hDLDJDQUFDLENBQUMsSUFBSSxFQUFFLEVBQUUsV0FBVyxFQUFFLFdBQVcsRUFBSyxDQUFDLEVBQ3hDLDJDQUFDLENBQUMsSUFBSSxFQUFFLEVBQUUsV0FBVyxFQUFFLFVBQVUsRUFBTSxDQUFDLENBQ3pDLENBQUM7UUFDRixLQUFLLE1BQU0sSUFBSSxJQUFJLE1BQU0sQ0FBQyxJQUFJLENBQUMsS0FBSyxDQUFDLEVBQUU7WUFDckMsSUFBSSxDQUFDLE1BQU0sQ0FBQyxJQUFJLEVBQUUsS0FBSyxDQUFDLElBQUksQ0FBQyxDQUFDO1NBQy9CO0lBQ0gsQ0FBQztJQUVELE1BQU0sQ0FBRSxJQUFZLEVBQUUsSUFBVTtRQUM5QixJQUFJLFFBQVE7WUFBRSxPQUFNO1FBQ3BCLE1BQU0sR0FBRyxHQUFHLGdEQUFNLENBQUMsSUFBSSxDQUFDLElBQUksRUFBRSwyQ0FBQyxDQUFDLElBQUksQ0FBQyxDQUFDO1FBQ3RDLE1BQU0sTUFBTSxHQUFRLDJDQUFDLENBQUMsSUFBSSxFQUFFLEVBQUUsU0FBUyxFQUFFLFFBQVEsRUFBRSxDQUFDLEVBQzlDLFdBQVcsR0FBRyxnREFBTSxDQUFDLE1BQU0sRUFBRSwyQ0FBQyxDQUFDLFFBQVEsRUFBRTtZQUN6QixXQUFXLEVBQUUsR0FBRztZQUNoQixPQUFPLEVBQUUsR0FBRyxFQUFFLENBQUMsSUFBSSxDQUFDLFFBQVEsQ0FBQyxHQUFHLENBQUM7U0FDbEMsQ0FBQyxDQUFDLEVBQ2pCLFdBQVcsR0FBRyxnREFBTSxDQUFDLE1BQU0sRUFBRSwyQ0FBQyxDQUFDLE1BQU0sRUFBRTtZQUN2QixXQUFXLEVBQUUsRUFBRTtTQUNoQixDQUFDLENBQUMsRUFDakIsVUFBVSxHQUFJLGdEQUFNLENBQUMsTUFBTSxFQUFFLDJDQUFDLENBQUMsUUFBUSxFQUFFO1lBQ3pCLFdBQVcsRUFBRSxHQUFHO1lBQ2hCLE9BQU8sRUFBRSxHQUFHLEVBQUUsQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDLEdBQUcsQ0FBQztTQUM5QixDQUFDLENBQUM7UUFDdkIsTUFBTSxJQUFJLEdBQUcsSUFBSSxDQUFDLElBQUksQ0FBQyxJQUFJLENBQUMsR0FBRztZQUM3QixJQUFJLEVBQVUsZ0RBQU0sQ0FBQyxHQUFHLEVBQUUsMkNBQUMsQ0FBQyxJQUFJLEVBQUUsRUFBRSxLQUFLLEVBQUUsa0JBQWtCLEVBQUUsV0FBVyxFQUFFLElBQUksRUFBRSxDQUFDLENBQUM7WUFDcEYsV0FBVyxFQUFHLGdEQUFNLENBQUMsR0FBRyxFQUFFLDJDQUFDLENBQUMsSUFBSSxDQUFDLENBQUM7WUFDbEMsR0FBRyxFQUFXLGdEQUFNLENBQUMsR0FBRyxFQUFFLDJDQUFDLENBQUMsSUFBSSxDQUFDLENBQUM7WUFDbEMsTUFBTSxFQUFRLGdEQUFNLENBQUMsR0FBRyxFQUFFLE1BQU0sQ0FBQztZQUNqQyxXQUFXLEVBQUUsV0FBVyxFQUFFLFVBQVU7WUFDcEMsUUFBUSxFQUFNLGdEQUFNLENBQUMsR0FBRyxFQUFFLDJDQUFDLENBQUMsSUFBSSxDQUFDLENBQUM7WUFDbEMsS0FBSyxFQUFTLGdEQUFNLENBQUMsR0FBRyxFQUFFLDJDQUFDLENBQUMsSUFBSSxDQUFDLENBQUM7WUFDbEMsTUFBTSxFQUFRLGdEQUFNLENBQUMsR0FBRyxFQUFFLDJDQUFDLENBQUMsSUFBSSxDQUFDLENBQUM7WUFDbEMsT0FBTyxFQUFPLGdEQUFNLENBQUMsR0FBRyxFQUFFLDJDQUFDLENBQUMsSUFBSSxDQUFDLENBQUM7WUFDbEMsU0FBUyxFQUFLLGdEQUFNLENBQUMsR0FBRyxFQUFFLDJDQUFDLENBQUMsSUFBSSxFQUFFLEVBQUUsU0FBUyxFQUFFLFdBQVcsRUFBRSxPQUFPLEVBQUUsR0FBRyxFQUFFLEdBQUUsSUFBSSxDQUFDLEtBQUssRUFBRSxHQUFDLEVBQUUsQ0FBQyxDQUFDO1lBQzdGLFFBQVEsRUFBTSxnREFBTSxDQUFDLEdBQUcsRUFBRSwyQ0FBQyxDQUFDLElBQUksQ0FBQyxDQUFDO1NBQ25DO1FBQ0QsSUFBSSxDQUFDLFNBQVMsQ0FBQyxLQUFLLENBQUMsVUFBVSxHQUFHLE1BQU07UUFDeEMsZ0RBQU0sQ0FBQyxJQUFJLENBQUMsSUFBSSxFQUFFLEdBQUcsQ0FBQztRQUN0QixPQUFPLElBQUk7SUFDYixDQUFDO0lBRUQsTUFBTSxDQUFFLElBQVU7UUFDaEIsSUFBSSxRQUFRO1lBQUUsT0FBTTtRQUNwQixJQUFJLENBQUMsSUFBSSxDQUFDLElBQUksQ0FBQyxJQUFJLENBQUMsQ0FBQyxXQUFXLENBQUMsV0FBVztZQUMxQywwREFBYyxDQUFDLElBQUksQ0FBQyxXQUFXLENBQUM7UUFDbEMsSUFBSSxDQUFDLElBQUksQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDLENBQUMsV0FBVyxDQUFDLFdBQVc7WUFDMUMsMERBQWMsQ0FBQyxJQUFJLENBQUMsTUFBTSxDQUFDO1FBQzdCLElBQUksQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDLElBQUksQ0FBQyxDQUFDLFFBQVEsQ0FBQyxXQUFXO1lBQ3ZDLDBEQUFjLENBQUMsSUFBSSxDQUFDLFFBQVEsQ0FBQztRQUMvQixJQUFJLENBQUMsSUFBSSxDQUFDLElBQUksQ0FBQyxJQUFJLENBQUMsQ0FBQyxLQUFLLENBQUMsV0FBVztZQUNwQyw2REFBaUIsQ0FBQyxJQUFJLENBQUMsS0FBSyxDQUFDO1FBQy9CLElBQUksQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDLElBQUksQ0FBQyxDQUFDLEdBQUcsQ0FBQyxXQUFXO1lBQ2xDLDBEQUFjLENBQUMsSUFBSSxDQUFDLEdBQUcsQ0FBQztRQUMxQixJQUFJLENBQUMsSUFBSSxDQUFDLElBQUksQ0FBQyxJQUFJLENBQUMsQ0FBQyxNQUFNLENBQUMsV0FBVztZQUNyQywwREFBYyxDQUFDLElBQUksQ0FBQyxNQUFNLENBQUM7UUFDN0IsSUFBSSxDQUFDLElBQUksQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDLENBQUMsT0FBTyxDQUFDLFdBQVc7WUFDdEMsMERBQWMsQ0FBQyxJQUFJLENBQUMsT0FBTyxDQUFDO1FBQzlCLElBQUksQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDLElBQUksQ0FBQyxDQUFDLFNBQVMsQ0FBQyxXQUFXO1lBQ3hDLDBEQUFjLENBQUMsSUFBSSxDQUFDLFNBQVMsQ0FBQztRQUNoQyxJQUFJLENBQUMsSUFBSSxDQUFDLElBQUksQ0FBQyxJQUFJLENBQUMsQ0FBQyxRQUFRLENBQUMsV0FBVztZQUN2QywwREFBYyxDQUFDLElBQUksQ0FBQyxRQUFRLENBQUM7UUFFL0IsTUFBTSxDQUFDLElBQUksRUFBRSxNQUFNLENBQUMsR0FBRyxJQUFJLENBQUMsTUFBTSxFQUFFO1FBQ3BDLElBQUksQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDLElBQUksQ0FBQyxDQUFDLE1BQU0sQ0FBQyxLQUFLLENBQUMsZUFBZTtZQUNqRCxJQUFJLENBQUMsSUFBSSxDQUFDLElBQUksQ0FBQyxJQUFJLENBQUMsQ0FBQyxPQUFPLENBQUMsS0FBSyxDQUFDLGVBQWU7Z0JBQ2xELElBQUksQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDLElBQUksQ0FBQyxDQUFDLFNBQVMsQ0FBQyxLQUFLLENBQUMsZUFBZTtvQkFDbEQsSUFBSTtRQUNOLElBQUksQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDLElBQUksQ0FBQyxDQUFDLFNBQVMsQ0FBQyxLQUFLLENBQUMsS0FBSztZQUN4QyxNQUFNO0lBQ1YsQ0FBQztDQUNGO0FBR00sTUFBTSxRQUFRO0lBUW5CLFlBQWEsS0FBYSxFQUFFLEtBQWE7UUFKekMsVUFBSyxHQUFVLEVBQUUsQ0FBQztRQUNsQixVQUFLLEdBQVcsQ0FBQyxDQUFDO1FBSWhCLElBQUksQ0FBQyxLQUFLLEdBQUksS0FBSztRQUNuQixJQUFJLENBQUMsSUFBSSxHQUFLLDJDQUFDLENBQUMsS0FBSyxFQUFFLEVBQUUsU0FBUyxFQUFFLE9BQU8sS0FBSyxFQUFFLEVBQUUsQ0FBQztRQUNyRCxJQUFJLENBQUMsTUFBTSxHQUFHLGdEQUFNLENBQUMsSUFBSSxDQUFDLElBQUksRUFBRSwyQ0FBQyxDQUFDLFFBQVEsRUFBRSxFQUFFLEtBQUssRUFBRSxDQUFDLEVBQUUsTUFBTSxFQUFFLENBQUMsRUFBRSxDQUFDLENBQXNCO0lBQzVGLENBQUM7SUFFRCxHQUFHLENBQUUsSUFBVTtRQUNiLElBQUksQ0FBQyxLQUFLLENBQUMsSUFBSSxDQUFDLElBQUksQ0FBQyxHQUFHLElBQUk7SUFDOUIsQ0FBQztJQUVELE1BQU0sQ0FBRSxJQUFVO1FBQ2hCLE9BQU8sSUFBSSxDQUFDLEtBQUssQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDO0lBQzlCLENBQUM7SUFFRCxNQUFNO1FBQ0osSUFBSSxDQUFDLE1BQU0sQ0FBQyxLQUFLLEdBQUcsSUFBSSxDQUFDLE1BQU0sQ0FBQyxNQUFNLEdBQUcsQ0FBQztRQUMxQyxNQUFNLElBQUksR0FBRyxJQUFJLENBQUMsR0FBRyxDQUFDLElBQUksQ0FBQyxJQUFJLENBQUMsV0FBVyxFQUFFLElBQUksQ0FBQyxJQUFJLENBQUMsWUFBWSxDQUFDO1FBQ3BFLElBQUksQ0FBQyxNQUFNLENBQUMsS0FBSyxHQUFHLElBQUksQ0FBQyxNQUFNLENBQUMsTUFBTSxHQUFHLElBQUk7UUFDN0MsSUFBSSxDQUFDLE1BQU0sRUFBRTtJQUNmLENBQUM7SUFFRCxNQUFNO1FBQ0oscUJBQXFCLENBQUMsR0FBRSxFQUFFO1lBQ3hCLHNDQUFzQztZQUN0QyxvQkFBb0I7WUFDcEIsTUFBTSxNQUFNLEdBQVcsRUFBRTtZQUN6QixJQUFJLEtBQUssR0FBVyxDQUFDO1lBQ3JCLEtBQUssTUFBTSxJQUFJLElBQUksTUFBTSxDQUFDLE1BQU0sQ0FBQyxJQUFJLENBQUMsS0FBSyxDQUFDLEVBQUU7Z0JBQzVDLE1BQU0sS0FBSyxHQUFJLElBQVksQ0FBQyxJQUFJLENBQUMsS0FBSyxDQUFDO2dCQUN2QyxJQUFJLEtBQUssRUFBRTtvQkFDVCxLQUFLLElBQUksS0FBSztvQkFDZCxNQUFNLENBQUMsSUFBSSxDQUFDLElBQUksQ0FBQyxHQUFHLEtBQUs7aUJBQUU7YUFBRTtZQUNqQyxJQUFJLEtBQUssS0FBSyxDQUFDO2dCQUFFLE9BQU07WUFFdkIsaUJBQWlCO1lBQ2pCLE1BQU0sRUFBQyxLQUFLLEVBQUUsTUFBTSxFQUFDLEdBQUcsSUFBSSxDQUFDLE1BQU07WUFDbkMsTUFBTSxPQUFPLEdBQUcsSUFBSSxDQUFDLE1BQU0sQ0FBQyxVQUFVLENBQUMsSUFBSSxDQUE2QixDQUFDO1lBRXpFLFFBQVE7WUFDUixPQUFPLENBQUMsU0FBUyxHQUFHLFNBQVM7WUFDN0IsT0FBTyxDQUFDLFFBQVEsQ0FBQyxDQUFDLEVBQUUsQ0FBQyxFQUFFLEtBQUssR0FBQyxDQUFDLEVBQUUsTUFBTSxHQUFDLENBQUMsQ0FBQztZQUV6QyxnQkFBZ0I7WUFDaEIsTUFBTSxPQUFPLEdBQUcsS0FBSyxHQUFJLENBQUM7WUFDMUIsTUFBTSxPQUFPLEdBQUcsTUFBTSxHQUFHLENBQUM7WUFDMUIsTUFBTSxNQUFNLEdBQUksT0FBTyxHQUFHLElBQUk7WUFFOUIscUJBQXFCO1lBQ3JCLElBQUksS0FBSyxHQUFHLENBQUM7WUFDYixLQUFLLE1BQU0sSUFBSSxJQUFJLE1BQU0sQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDLEtBQUssQ0FBQyxDQUFDLElBQUksRUFBRSxFQUFFO2dCQUNqRCxNQUFNLEtBQUssR0FBRyxNQUFNLENBQUMsSUFBSSxDQUFDO2dCQUMxQixJQUFJLEtBQUssRUFBRTtvQkFDVCxNQUFNLE9BQU8sR0FBRyxLQUFLLEdBQUcsS0FBSztvQkFDN0IsTUFBTSxHQUFHLEdBQU8sS0FBSyxHQUFHLENBQUMsQ0FBQyxHQUFDLE9BQU8sQ0FBQztvQkFDbkMsT0FBTyxDQUFDLFNBQVMsRUFBRTtvQkFDbkIsT0FBTyxDQUFDLE1BQU0sQ0FBQyxPQUFPLEVBQUUsT0FBTyxDQUFDO29CQUNoQyxPQUFPLENBQUMsR0FBRyxDQUFDLE9BQU8sRUFBRSxPQUFPLEVBQUUsTUFBTSxFQUFFLEtBQUssR0FBRyxJQUFJLENBQUMsRUFBRSxFQUFFLEdBQUcsR0FBRyxJQUFJLENBQUMsRUFBRSxDQUFDO29CQUNyRSxrQ0FBa0M7b0JBQ2xDLE1BQU0sQ0FBQyxTQUFTLEVBQUUsV0FBVyxDQUFDLEdBQUcsSUFBSSxDQUFDLEtBQUssQ0FBQyxJQUFJLENBQUMsQ0FBQyxNQUFNLEVBQUU7b0JBQzFELE9BQU8sQ0FBQyxTQUFTLEdBQUcsU0FBUztvQkFDN0IsT0FBTyxDQUFDLFNBQVMsR0FBRyxHQUFHO29CQUN2QixPQUFPLENBQUMsV0FBVyxHQUFHLFdBQVcsbUNBQWlDO29CQUNsRSxPQUFPLENBQUMsSUFBSSxFQUFFO29CQUNkLE9BQU8sQ0FBQyxNQUFNLEVBQUU7b0JBQ2hCLEtBQUssR0FBRyxHQUFHO2lCQUFFO2FBQUU7UUFBQyxDQUFDLENBQUM7SUFBQyxDQUFDO0NBQUU7QUFFekIsTUFBTSxlQUFlO0lBVTFCO1FBTkEsVUFBSyxHQUFVLEVBQUUsQ0FBQztRQU9oQixJQUFJLENBQUMsSUFBSSxHQUFLLDJDQUFDLENBQUMsS0FBSyxFQUFFLEVBQUUsU0FBUyxFQUFFLGFBQWEsRUFBRSxDQUFDO1FBQ3BELElBQUksQ0FBQyxNQUFNLEdBQUcsZ0RBQU0sQ0FBQyxJQUFJLENBQUMsSUFBSSxFQUFFLDJDQUFDLENBQUMsUUFBUSxFQUFFLEVBQUUsS0FBSyxFQUFFLENBQUMsRUFBRSxNQUFNLEVBQUUsQ0FBQyxFQUFFLENBQUMsQ0FBc0I7SUFBQyxDQUFDO0lBUDlGLEdBQUcsQ0FBRSxJQUFVO1FBQ2IsSUFBSSxDQUFDLEtBQUssQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDLEdBQUcsSUFBSTtJQUFDLENBQUM7SUFDaEMsTUFBTSxDQUFFLElBQVU7UUFDaEIsT0FBTyxJQUFJLENBQUMsS0FBSyxDQUFDLElBQUksQ0FBQyxJQUFJLENBQUM7SUFBQyxDQUFDO0lBTWhDLE1BQU07UUFDSixJQUFJLENBQUMsTUFBTSxDQUFDLEtBQUssR0FBRyxJQUFJLENBQUMsTUFBTSxDQUFDLE1BQU0sR0FBRyxDQUFDO1FBQzFDLE1BQU0sSUFBSSxHQUFHLElBQUksQ0FBQyxHQUFHLENBQUMsSUFBSSxDQUFDLElBQUksQ0FBQyxXQUFXLEVBQUUsSUFBSSxDQUFDLElBQUksQ0FBQyxZQUFZLENBQUM7UUFDcEUsSUFBSSxDQUFDLE1BQU0sQ0FBQyxLQUFLLEdBQUcsSUFBSSxDQUFDLE1BQU0sQ0FBQyxNQUFNLEdBQUcsSUFBSTtRQUM3QyxJQUFJLENBQUMsTUFBTSxFQUFFO0lBQUMsQ0FBQztJQUVqQixNQUFNO1FBQ0oscUJBQXFCLENBQUMsR0FBRSxFQUFFO1lBQ3hCLHNDQUFzQztZQUN0QyxvQkFBb0I7WUFDcEIsSUFBSSxLQUFLLEdBQVcsQ0FBQztZQUNyQixLQUFLLE1BQU0sSUFBSSxJQUFJLE1BQU0sQ0FBQyxNQUFNLENBQUMsSUFBSSxDQUFDLEtBQUssQ0FBQyxFQUFFO2dCQUM1QyxLQUFLLElBQUksSUFBSSxDQUFDLFFBQVE7YUFDdkI7WUFDRCxJQUFJLEtBQUssS0FBSyxDQUFDO2dCQUFFLE9BQU07WUFFdkIsaUJBQWlCO1lBQ2pCLE1BQU0sRUFBQyxLQUFLLEVBQUUsTUFBTSxFQUFDLEdBQUcsSUFBSSxDQUFDLE1BQU07WUFDbkMsTUFBTSxPQUFPLEdBQUcsSUFBSSxDQUFDLE1BQU0sQ0FBQyxVQUFVLENBQUMsSUFBSSxDQUE2QixDQUFDO1lBRXpFLFFBQVE7WUFDUixPQUFPLENBQUMsU0FBUyxHQUFHLFNBQVM7WUFDN0IsT0FBTyxDQUFDLFFBQVEsQ0FBQyxDQUFDLEVBQUUsQ0FBQyxFQUFFLEtBQUssR0FBQyxDQUFDLEVBQUUsTUFBTSxHQUFDLENBQUMsQ0FBQztZQUV6QyxnQkFBZ0I7WUFDaEIsTUFBTSxPQUFPLEdBQUcsS0FBSyxHQUFJLENBQUM7WUFDMUIsTUFBTSxPQUFPLEdBQUcsTUFBTSxHQUFHLENBQUM7WUFDMUIsTUFBTSxNQUFNLEdBQUksT0FBTyxHQUFHLElBQUk7WUFFOUIscUJBQXFCO1lBQ3JCLElBQUksS0FBSyxHQUFHLENBQUM7WUFDYixLQUFLLE1BQU0sSUFBSSxJQUFJLE1BQU0sQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDLEtBQUssQ0FBQyxDQUFDLElBQUksRUFBRSxFQUFFO2dCQUNqRCxNQUFNLElBQUksR0FBRyxJQUFJLENBQUMsS0FBSyxDQUFDLElBQUksQ0FBQztnQkFDN0IsSUFBSSxJQUFJLENBQUMsUUFBUSxLQUFLLENBQUM7b0JBQUUsU0FBUTtnQkFDakMsTUFBTSxPQUFPLEdBQUcsSUFBSSxDQUFDLFFBQVEsR0FBRyxLQUFLO2dCQUNyQyxNQUFNLEdBQUcsR0FBTyxLQUFLLEdBQUcsQ0FBQyxDQUFDLEdBQUMsT0FBTyxDQUFDO2dCQUNuQyxPQUFPLENBQUMsU0FBUyxFQUFFO2dCQUNuQixPQUFPLENBQUMsTUFBTSxDQUFDLE9BQU8sRUFBRSxPQUFPLENBQUM7Z0JBQ2hDLE9BQU8sQ0FBQyxHQUFHLENBQUMsT0FBTyxFQUFFLE9BQU8sRUFBRSxNQUFNLEVBQUUsS0FBSyxHQUFHLElBQUksQ0FBQyxFQUFFLEVBQUUsR0FBRyxHQUFHLElBQUksQ0FBQyxFQUFFLENBQUM7Z0JBQ3JFLGtDQUFrQztnQkFDbEMsTUFBTSxDQUFDLFNBQVMsRUFBRSxXQUFXLENBQUMsR0FBRyxJQUFJLENBQUMsTUFBTSxFQUFFO2dCQUM5QyxPQUFPLENBQUMsU0FBUyxHQUFHLFNBQVM7Z0JBQzdCLE9BQU8sQ0FBQyxXQUFXLEdBQUcsV0FBVyxtQ0FBaUM7Z0JBQ2xFLDhDQUE4QztnQkFDOUMsT0FBTyxDQUFDLFNBQVMsR0FBRyxHQUFHO2dCQUN2QixPQUFPLENBQUMsSUFBSSxFQUFFO2dCQUNkLE9BQU8sQ0FBQyxNQUFNLEVBQUU7Z0JBQ2hCLEtBQUssR0FBRyxHQUFHO2FBQUU7UUFBQyxDQUFDLENBQUM7SUFBQyxDQUFDO0NBRXpCOzs7Ozs7O1VDeFREO1VBQ0E7O1VBRUE7VUFDQTtVQUNBO1VBQ0E7VUFDQTtVQUNBO1VBQ0E7VUFDQTtVQUNBO1VBQ0E7VUFDQTtVQUNBO1VBQ0E7O1VBRUE7VUFDQTs7VUFFQTtVQUNBO1VBQ0E7O1VBRUE7VUFDQTs7Ozs7V0N6QkE7V0FDQTtXQUNBO1dBQ0E7V0FDQTtXQUNBLGdDQUFnQyxZQUFZO1dBQzVDO1dBQ0EsRTs7Ozs7V0NQQTtXQUNBO1dBQ0E7V0FDQTtXQUNBLHdDQUF3Qyx5Q0FBeUM7V0FDakY7V0FDQTtXQUNBLEU7Ozs7O1dDUEE7V0FDQTtXQUNBO1dBQ0E7V0FDQSxFQUFFO1dBQ0Y7V0FDQTtXQUNBLENBQUMsSTs7Ozs7V0NQRCx3Rjs7Ozs7V0NBQTtXQUNBO1dBQ0E7V0FDQSxzREFBc0Qsa0JBQWtCO1dBQ3hFO1dBQ0EsK0NBQStDLGNBQWM7V0FDN0QsRTs7Ozs7V0NOQTtXQUNBO1dBQ0E7V0FDQTtXQUNBO1dBQ0E7V0FDQTtXQUNBO1dBQ0E7V0FDQTtXQUNBO1dBQ0E7V0FDQTtXQUNBO1dBQ0E7V0FDQSxrQzs7Ozs7V0NmQTs7V0FFQTtXQUNBO1dBQ0E7V0FDQTtXQUNBO1dBQ0E7O1dBRUE7O1dBRUE7O1dBRUE7O1dBRUE7O1dBRUE7O1dBRUE7O1dBRUEsb0I7Ozs7Ozs7Ozs7Ozs7Ozs7QUNyQjhCO0FBQzZDO0FBQ0M7QUFDRTtBQUNHO0FBQ2pGLGtEQUFrRDtBQUNGO0FBRWhELFFBQVEsQ0FBQyxJQUFJLENBQUMsU0FBUyxHQUFHLDBCQUEwQjtBQUVwRCxvR0FBb0c7QUFDcEcsTUFBTSxlQUFlLEdBQUksQ0FBQztBQUMxQixNQUFNLFVBQVUsR0FBUyxLQUFLO0FBQzlCLE1BQU0sZ0JBQWdCLEdBQUcsS0FBSztBQUU5QixpRUFBUSxFQUFFLENBQUMsSUFBSSxDQUFDLEdBQUUsRUFBRTtJQUNsQixRQUFRLENBQUMsSUFBSSxDQUFDLE9BQU8sR0FBRyxHQUFHLEVBQUU7UUFDM0IsUUFBUSxDQUFDLElBQUksQ0FBQyxTQUFTLEdBQUcsRUFBRTtRQUM1QixRQUFRLENBQUMsSUFBSSxDQUFDLE9BQU8sR0FBRyxJQUFJO1FBQzVCLEtBQUssRUFBRTtJQUNULENBQUM7SUFDRCxRQUFRLENBQUMsSUFBSSxDQUFDLFNBQVMsR0FBRyxpQ0FBaUM7QUFDN0QsQ0FBQyxDQUFDO0FBRUYsU0FBUyxLQUFLO0lBRVosa0dBQWtHO0lBQ2xHLE1BQU0sRUFBRSxHQUFHO1FBQ1QsR0FBRyxFQUFNLElBQUksbURBQUcsRUFBRTtRQUNsQixLQUFLLEVBQUksSUFBSSxxREFBSyxFQUFFO1FBQ3BCLE9BQU8sRUFBRSxJQUFJLHdEQUFRLENBQUMsd0JBQXdCLEVBQUcsUUFBUSxDQUFDO1FBQzFELE9BQU8sRUFBRSxJQUFJLCtEQUFlLEVBQUU7S0FDL0I7SUFFRCxrR0FBa0c7SUFDbEcsTUFBTSxJQUFJLEdBQUcsSUFBSSw4REFBSSxDQUFDLEVBQUUsQ0FBQztJQUN6QixNQUFNLEtBQUssR0FBVSxFQUFFO0lBQ3ZCLEtBQUssSUFBSSxDQUFDLEdBQUcsQ0FBQyxFQUFFLENBQUMsR0FBRywrREFBUyxFQUFFLENBQUMsRUFBRSxFQUFFO1FBQ2xDLE1BQU0sSUFBSSxHQUFNLE9BQU8sQ0FBQyxFQUFFO1FBQzFCLE1BQU0sT0FBTyxHQUFHLElBQUksQ0FBQyxLQUFLLENBQUMsSUFBSSxDQUFDLE1BQU0sRUFBRSxHQUFDLGlFQUFXLENBQUM7UUFDckQsS0FBSyxDQUFDLElBQUksQ0FBQyxHQUFLLElBQUksOERBQUksQ0FBQyxFQUFFLEVBQUUsSUFBSSxFQUFFLElBQUksRUFBRSxPQUFPLENBQUM7S0FDbEQ7SUFFRCxrR0FBa0c7SUFDbEcsS0FBSyxNQUFNLEVBQUUsSUFBSSxNQUFNLENBQUMsTUFBTSxDQUFDLEVBQUUsQ0FBQyxFQUFFO1FBQ2xDLDBEQUFNLENBQUMsUUFBUSxDQUFDLElBQUksRUFBRSxFQUFFLENBQUMsSUFBSSxDQUFDO0tBQy9CO0lBRUQsa0dBQWtHO0lBQ2xHLEVBQUUsQ0FBQyxLQUFLLENBQUMsSUFBSSxDQUFDLEtBQUssQ0FBQztJQUVwQixrR0FBa0c7SUFDbEcsTUFBTSxFQUFFO0lBQ1IsTUFBTSxDQUFDLGdCQUFnQixDQUFDLFFBQVEsRUFBRSw0REFBUSxDQUFDLEdBQUcsRUFBRSxNQUFNLENBQUMsQ0FBQztJQUV4RCxrR0FBa0c7SUFDbEcsTUFBTSxFQUFFO0lBQ1IsU0FBUyxNQUFNO1FBQ2IsZ0dBQWdHO1FBQ2hHLHlEQUFHLEVBQUU7UUFDTCxJQUFJLENBQUMsUUFBUSxDQUFDLEtBQUssR0FBRyx5REFBRztRQUV6QixnR0FBZ0c7UUFDaEcsSUFBSSxDQUFDLE1BQU0sRUFBRTtRQUViLGdHQUFnRztRQUNoRyxNQUFNLFFBQVEsR0FBZ0IsRUFBRTtRQUNoQyxLQUFLLE1BQU0sSUFBSSxJQUFJLE1BQU0sQ0FBQyxNQUFNLENBQUMsS0FBSyxDQUFDLEVBQUU7WUFDdkMsSUFBSSxDQUFDLE1BQU0sRUFBRTtZQUNiLElBQUksSUFBSSxDQUFDLFNBQVMsR0FBRyxDQUFDO2dCQUFFLFFBQVEsQ0FBQyxJQUFJLENBQUMsSUFBWSxDQUFDO1NBQ3BEO1FBRUQsZ0dBQWdHO1FBQ2hHLElBQUksZ0JBQWdCLEVBQUU7WUFDcEIsTUFBTSxJQUFJLEdBQUcsOERBQVUsQ0FBQyxNQUFNLENBQUMsTUFBTSxDQUFDLEtBQUssQ0FBQyxDQUFDO1lBQzdDLDhEQUFVLENBQUM7Z0JBQ1QsQ0FBQyxNQUFhLEVBQUMsRUFBRSxLQUFJLENBQUMsSUFBSSxDQUFDLE1BQU0sQ0FBQztnQkFDbEMsQ0FBQyxNQUFhLEVBQUMsRUFBRSxLQUFJLENBQUMsUUFBUSxDQUFDLE1BQU0sQ0FBQzthQUN2QyxDQUFDLENBQUMsMERBQU0sQ0FBQyxJQUFJLENBQUMsT0FBTyxDQUFDLENBQUM7U0FDekI7UUFFRCxnR0FBZ0c7UUFDaEcsSUFBSSxVQUFVLElBQUksUUFBUSxDQUFDLE1BQU0sR0FBRyxDQUFDLEVBQUU7WUFDckMsTUFBTSxRQUFRLEdBQUcsOERBQVUsQ0FBQyxRQUFRLENBQUM7WUFDckMsUUFBUSxDQUFDLEtBQUssRUFBRTtTQUNqQjtRQUVELGdHQUFnRztRQUNoRyxLQUFLLE1BQU0sS0FBSyxJQUFJLENBQUMsRUFBRSxDQUFDLE9BQU8sRUFBQyxFQUFFLENBQUMsT0FBTyxDQUFDLEVBQUU7WUFDM0MsS0FBSyxDQUFDLE1BQU0sRUFBRTtTQUNmO1FBRUQsZ0dBQWdHO1FBQ2hHLHlEQUFLLENBQUMsZUFBZSxFQUFFLE1BQU0sQ0FBQztJQUNoQyxDQUFDO0lBRUQsa0dBQWtHO0lBQ2xHLFNBQVMsTUFBTTtRQUNiLEVBQUUsQ0FBQyxPQUFPLENBQUMsTUFBTSxFQUFFO1FBQ25CLEVBQUUsQ0FBQyxPQUFPLENBQUMsTUFBTSxFQUFFO0lBQ3JCLENBQUM7QUFDSCxDQUFDIiwiZmlsZSI6Im1haW4uanMiLCJzb3VyY2VzQ29udGVudCI6WyJcbmxldCB3YXNtO1xuXG5sZXQgY2FjaGVkVGV4dERlY29kZXIgPSBuZXcgVGV4dERlY29kZXIoJ3V0Zi04JywgeyBpZ25vcmVCT006IHRydWUsIGZhdGFsOiB0cnVlIH0pO1xuXG5jYWNoZWRUZXh0RGVjb2Rlci5kZWNvZGUoKTtcblxubGV0IGNhY2hlZ2V0VWludDhNZW1vcnkwID0gbnVsbDtcbmZ1bmN0aW9uIGdldFVpbnQ4TWVtb3J5MCgpIHtcbiAgICBpZiAoY2FjaGVnZXRVaW50OE1lbW9yeTAgPT09IG51bGwgfHwgY2FjaGVnZXRVaW50OE1lbW9yeTAuYnVmZmVyICE9PSB3YXNtLm1lbW9yeS5idWZmZXIpIHtcbiAgICAgICAgY2FjaGVnZXRVaW50OE1lbW9yeTAgPSBuZXcgVWludDhBcnJheSh3YXNtLm1lbW9yeS5idWZmZXIpO1xuICAgIH1cbiAgICByZXR1cm4gY2FjaGVnZXRVaW50OE1lbW9yeTA7XG59XG5cbmZ1bmN0aW9uIGdldFN0cmluZ0Zyb21XYXNtMChwdHIsIGxlbikge1xuICAgIHJldHVybiBjYWNoZWRUZXh0RGVjb2Rlci5kZWNvZGUoZ2V0VWludDhNZW1vcnkwKCkuc3ViYXJyYXkocHRyLCBwdHIgKyBsZW4pKTtcbn1cblxuY29uc3QgaGVhcCA9IG5ldyBBcnJheSgzMikuZmlsbCh1bmRlZmluZWQpO1xuXG5oZWFwLnB1c2godW5kZWZpbmVkLCBudWxsLCB0cnVlLCBmYWxzZSk7XG5cbmxldCBoZWFwX25leHQgPSBoZWFwLmxlbmd0aDtcblxuZnVuY3Rpb24gYWRkSGVhcE9iamVjdChvYmopIHtcbiAgICBpZiAoaGVhcF9uZXh0ID09PSBoZWFwLmxlbmd0aCkgaGVhcC5wdXNoKGhlYXAubGVuZ3RoICsgMSk7XG4gICAgY29uc3QgaWR4ID0gaGVhcF9uZXh0O1xuICAgIGhlYXBfbmV4dCA9IGhlYXBbaWR4XTtcblxuICAgIGlmICh0eXBlb2YoaGVhcF9uZXh0KSAhPT0gJ251bWJlcicpIHRocm93IG5ldyBFcnJvcignY29ycnVwdCBoZWFwJyk7XG5cbiAgICBoZWFwW2lkeF0gPSBvYmo7XG4gICAgcmV0dXJuIGlkeDtcbn1cblxuZnVuY3Rpb24gZ2V0T2JqZWN0KGlkeCkgeyByZXR1cm4gaGVhcFtpZHhdOyB9XG5cbmZ1bmN0aW9uIGRyb3BPYmplY3QoaWR4KSB7XG4gICAgaWYgKGlkeCA8IDM2KSByZXR1cm47XG4gICAgaGVhcFtpZHhdID0gaGVhcF9uZXh0O1xuICAgIGhlYXBfbmV4dCA9IGlkeDtcbn1cblxuZnVuY3Rpb24gdGFrZU9iamVjdChpZHgpIHtcbiAgICBjb25zdCByZXQgPSBnZXRPYmplY3QoaWR4KTtcbiAgICBkcm9wT2JqZWN0KGlkeCk7XG4gICAgcmV0dXJuIHJldDtcbn1cblxuZnVuY3Rpb24gX2Fzc2VydE51bShuKSB7XG4gICAgaWYgKHR5cGVvZihuKSAhPT0gJ251bWJlcicpIHRocm93IG5ldyBFcnJvcignZXhwZWN0ZWQgYSBudW1iZXIgYXJndW1lbnQnKTtcbn1cblxubGV0IFdBU01fVkVDVE9SX0xFTiA9IDA7XG5cbmZ1bmN0aW9uIHBhc3NBcnJheThUb1dhc20wKGFyZywgbWFsbG9jKSB7XG4gICAgY29uc3QgcHRyID0gbWFsbG9jKGFyZy5sZW5ndGggKiAxKTtcbiAgICBnZXRVaW50OE1lbW9yeTAoKS5zZXQoYXJnLCBwdHIgLyAxKTtcbiAgICBXQVNNX1ZFQ1RPUl9MRU4gPSBhcmcubGVuZ3RoO1xuICAgIHJldHVybiBwdHI7XG59XG5cbmNvbnN0IHUzMkN2dFNoaW0gPSBuZXcgVWludDMyQXJyYXkoMik7XG5cbmNvbnN0IHVpbnQ2NEN2dFNoaW0gPSBuZXcgQmlnVWludDY0QXJyYXkodTMyQ3Z0U2hpbS5idWZmZXIpO1xuXG5sZXQgY2FjaGVnZXRJbnQzMk1lbW9yeTAgPSBudWxsO1xuZnVuY3Rpb24gZ2V0SW50MzJNZW1vcnkwKCkge1xuICAgIGlmIChjYWNoZWdldEludDMyTWVtb3J5MCA9PT0gbnVsbCB8fCBjYWNoZWdldEludDMyTWVtb3J5MC5idWZmZXIgIT09IHdhc20ubWVtb3J5LmJ1ZmZlcikge1xuICAgICAgICBjYWNoZWdldEludDMyTWVtb3J5MCA9IG5ldyBJbnQzMkFycmF5KHdhc20ubWVtb3J5LmJ1ZmZlcik7XG4gICAgfVxuICAgIHJldHVybiBjYWNoZWdldEludDMyTWVtb3J5MDtcbn1cblxuZnVuY3Rpb24gZ2V0QXJyYXlVOEZyb21XYXNtMChwdHIsIGxlbikge1xuICAgIHJldHVybiBnZXRVaW50OE1lbW9yeTAoKS5zdWJhcnJheShwdHIgLyAxLCBwdHIgLyAxICsgbGVuKTtcbn1cbi8qKlxuKi9cbmV4cG9ydCBjbGFzcyBDb250cmFjdCB7XG5cbiAgICBzdGF0aWMgX193cmFwKHB0cikge1xuICAgICAgICBjb25zdCBvYmogPSBPYmplY3QuY3JlYXRlKENvbnRyYWN0LnByb3RvdHlwZSk7XG4gICAgICAgIG9iai5wdHIgPSBwdHI7XG5cbiAgICAgICAgcmV0dXJuIG9iajtcbiAgICB9XG5cbiAgICB0b0pTT04oKSB7XG4gICAgICAgIHJldHVybiB7XG4gICAgICAgICAgICBnZXRfYmxvY2s6IHRoaXMuZ2V0X2Jsb2NrLFxuICAgICAgICB9O1xuICAgIH1cblxuICAgIHRvU3RyaW5nKCkge1xuICAgICAgICByZXR1cm4gSlNPTi5zdHJpbmdpZnkodGhpcyk7XG4gICAgfVxuXG4gICAgX19kZXN0cm95X2ludG9fcmF3KCkge1xuICAgICAgICBjb25zdCBwdHIgPSB0aGlzLnB0cjtcbiAgICAgICAgdGhpcy5wdHIgPSAwO1xuXG4gICAgICAgIHJldHVybiBwdHI7XG4gICAgfVxuXG4gICAgZnJlZSgpIHtcbiAgICAgICAgY29uc3QgcHRyID0gdGhpcy5fX2Rlc3Ryb3lfaW50b19yYXcoKTtcbiAgICAgICAgd2FzbS5fX3diZ19jb250cmFjdF9mcmVlKHB0cik7XG4gICAgfVxuICAgIC8qKlxuICAgICovXG4gICAgY29uc3RydWN0b3IoKSB7XG4gICAgICAgIHZhciByZXQgPSB3YXNtLmNvbnRyYWN0X25ldygpO1xuICAgICAgICByZXR1cm4gQ29udHJhY3QuX193cmFwKHJldCk7XG4gICAgfVxuICAgIC8qKlxuICAgICogQHBhcmFtIHtVaW50OEFycmF5fSBzZW5kZXJcbiAgICAqL1xuICAgIHNldCBzZW5kZXIoc2VuZGVyKSB7XG4gICAgICAgIGlmICh0aGlzLnB0ciA9PSAwKSB0aHJvdyBuZXcgRXJyb3IoJ0F0dGVtcHQgdG8gdXNlIGEgbW92ZWQgdmFsdWUnKTtcbiAgICAgICAgX2Fzc2VydE51bSh0aGlzLnB0cik7XG4gICAgICAgIHZhciBwdHIwID0gcGFzc0FycmF5OFRvV2FzbTAoc2VuZGVyLCB3YXNtLl9fd2JpbmRnZW5fbWFsbG9jKTtcbiAgICAgICAgdmFyIGxlbjAgPSBXQVNNX1ZFQ1RPUl9MRU47XG4gICAgICAgIHdhc20uY29udHJhY3Rfc2V0X3NlbmRlcih0aGlzLnB0ciwgcHRyMCwgbGVuMCk7XG4gICAgfVxuICAgIC8qKlxuICAgICogQHBhcmFtIHtCaWdJbnR9IGhlaWdodFxuICAgICovXG4gICAgc2V0IGJsb2NrKGhlaWdodCkge1xuICAgICAgICBpZiAodGhpcy5wdHIgPT0gMCkgdGhyb3cgbmV3IEVycm9yKCdBdHRlbXB0IHRvIHVzZSBhIG1vdmVkIHZhbHVlJyk7XG4gICAgICAgIF9hc3NlcnROdW0odGhpcy5wdHIpO1xuICAgICAgICB1aW50NjRDdnRTaGltWzBdID0gaGVpZ2h0O1xuICAgICAgICBjb25zdCBsb3cwID0gdTMyQ3Z0U2hpbVswXTtcbiAgICAgICAgY29uc3QgaGlnaDAgPSB1MzJDdnRTaGltWzFdO1xuICAgICAgICB3YXNtLmNvbnRyYWN0X3NldF9ibG9jayh0aGlzLnB0ciwgbG93MCwgaGlnaDApO1xuICAgIH1cbiAgICAvKipcbiAgICAqIEByZXR1cm5zIHtCaWdJbnR9XG4gICAgKi9cbiAgICBnZXQgZ2V0X2Jsb2NrKCkge1xuICAgICAgICB0cnkge1xuICAgICAgICAgICAgaWYgKHRoaXMucHRyID09IDApIHRocm93IG5ldyBFcnJvcignQXR0ZW1wdCB0byB1c2UgYSBtb3ZlZCB2YWx1ZScpO1xuICAgICAgICAgICAgY29uc3QgcmV0cHRyID0gd2FzbS5fX3diaW5kZ2VuX2FkZF90b19zdGFja19wb2ludGVyKC0xNik7XG4gICAgICAgICAgICBfYXNzZXJ0TnVtKHRoaXMucHRyKTtcbiAgICAgICAgICAgIHdhc20uY29udHJhY3RfZ2V0X2Jsb2NrKHJldHB0ciwgdGhpcy5wdHIpO1xuICAgICAgICAgICAgdmFyIHIwID0gZ2V0SW50MzJNZW1vcnkwKClbcmV0cHRyIC8gNCArIDBdO1xuICAgICAgICAgICAgdmFyIHIxID0gZ2V0SW50MzJNZW1vcnkwKClbcmV0cHRyIC8gNCArIDFdO1xuICAgICAgICAgICAgdTMyQ3Z0U2hpbVswXSA9IHIwO1xuICAgICAgICAgICAgdTMyQ3Z0U2hpbVsxXSA9IHIxO1xuICAgICAgICAgICAgY29uc3QgbjAgPSB1aW50NjRDdnRTaGltWzBdO1xuICAgICAgICAgICAgcmV0dXJuIG4wO1xuICAgICAgICB9IGZpbmFsbHkge1xuICAgICAgICAgICAgd2FzbS5fX3diaW5kZ2VuX2FkZF90b19zdGFja19wb2ludGVyKDE2KTtcbiAgICAgICAgfVxuICAgIH1cbiAgICAvKipcbiAgICAqIEBwYXJhbSB7VWludDhBcnJheX0gcmVzcG9uc2VcbiAgICAqL1xuICAgIHNldCBuZXh0X3F1ZXJ5X3Jlc3BvbnNlKHJlc3BvbnNlKSB7XG4gICAgICAgIGlmICh0aGlzLnB0ciA9PSAwKSB0aHJvdyBuZXcgRXJyb3IoJ0F0dGVtcHQgdG8gdXNlIGEgbW92ZWQgdmFsdWUnKTtcbiAgICAgICAgX2Fzc2VydE51bSh0aGlzLnB0cik7XG4gICAgICAgIHZhciBwdHIwID0gcGFzc0FycmF5OFRvV2FzbTAocmVzcG9uc2UsIHdhc20uX193YmluZGdlbl9tYWxsb2MpO1xuICAgICAgICB2YXIgbGVuMCA9IFdBU01fVkVDVE9SX0xFTjtcbiAgICAgICAgd2FzbS5jb250cmFjdF9zZXRfbmV4dF9xdWVyeV9yZXNwb25zZSh0aGlzLnB0ciwgcHRyMCwgbGVuMCk7XG4gICAgfVxuICAgIC8qKlxuICAgICogQHBhcmFtIHtVaW50OEFycmF5fSBtc2dcbiAgICAqIEByZXR1cm5zIHtVaW50OEFycmF5fVxuICAgICovXG4gICAgaW5pdChtc2cpIHtcbiAgICAgICAgdHJ5IHtcbiAgICAgICAgICAgIGlmICh0aGlzLnB0ciA9PSAwKSB0aHJvdyBuZXcgRXJyb3IoJ0F0dGVtcHQgdG8gdXNlIGEgbW92ZWQgdmFsdWUnKTtcbiAgICAgICAgICAgIGNvbnN0IHJldHB0ciA9IHdhc20uX193YmluZGdlbl9hZGRfdG9fc3RhY2tfcG9pbnRlcigtMTYpO1xuICAgICAgICAgICAgX2Fzc2VydE51bSh0aGlzLnB0cik7XG4gICAgICAgICAgICB2YXIgcHRyMCA9IHBhc3NBcnJheThUb1dhc20wKG1zZywgd2FzbS5fX3diaW5kZ2VuX21hbGxvYyk7XG4gICAgICAgICAgICB2YXIgbGVuMCA9IFdBU01fVkVDVE9SX0xFTjtcbiAgICAgICAgICAgIHdhc20uY29udHJhY3RfaW5pdChyZXRwdHIsIHRoaXMucHRyLCBwdHIwLCBsZW4wKTtcbiAgICAgICAgICAgIHZhciByMCA9IGdldEludDMyTWVtb3J5MCgpW3JldHB0ciAvIDQgKyAwXTtcbiAgICAgICAgICAgIHZhciByMSA9IGdldEludDMyTWVtb3J5MCgpW3JldHB0ciAvIDQgKyAxXTtcbiAgICAgICAgICAgIHZhciB2MSA9IGdldEFycmF5VThGcm9tV2FzbTAocjAsIHIxKS5zbGljZSgpO1xuICAgICAgICAgICAgd2FzbS5fX3diaW5kZ2VuX2ZyZWUocjAsIHIxICogMSk7XG4gICAgICAgICAgICByZXR1cm4gdjE7XG4gICAgICAgIH0gZmluYWxseSB7XG4gICAgICAgICAgICB3YXNtLl9fd2JpbmRnZW5fYWRkX3RvX3N0YWNrX3BvaW50ZXIoMTYpO1xuICAgICAgICB9XG4gICAgfVxuICAgIC8qKlxuICAgICogQHBhcmFtIHtVaW50OEFycmF5fSBtc2dcbiAgICAqIEByZXR1cm5zIHtVaW50OEFycmF5fVxuICAgICovXG4gICAgaGFuZGxlKG1zZykge1xuICAgICAgICB0cnkge1xuICAgICAgICAgICAgaWYgKHRoaXMucHRyID09IDApIHRocm93IG5ldyBFcnJvcignQXR0ZW1wdCB0byB1c2UgYSBtb3ZlZCB2YWx1ZScpO1xuICAgICAgICAgICAgY29uc3QgcmV0cHRyID0gd2FzbS5fX3diaW5kZ2VuX2FkZF90b19zdGFja19wb2ludGVyKC0xNik7XG4gICAgICAgICAgICBfYXNzZXJ0TnVtKHRoaXMucHRyKTtcbiAgICAgICAgICAgIHZhciBwdHIwID0gcGFzc0FycmF5OFRvV2FzbTAobXNnLCB3YXNtLl9fd2JpbmRnZW5fbWFsbG9jKTtcbiAgICAgICAgICAgIHZhciBsZW4wID0gV0FTTV9WRUNUT1JfTEVOO1xuICAgICAgICAgICAgd2FzbS5jb250cmFjdF9oYW5kbGUocmV0cHRyLCB0aGlzLnB0ciwgcHRyMCwgbGVuMCk7XG4gICAgICAgICAgICB2YXIgcjAgPSBnZXRJbnQzMk1lbW9yeTAoKVtyZXRwdHIgLyA0ICsgMF07XG4gICAgICAgICAgICB2YXIgcjEgPSBnZXRJbnQzMk1lbW9yeTAoKVtyZXRwdHIgLyA0ICsgMV07XG4gICAgICAgICAgICB2YXIgdjEgPSBnZXRBcnJheVU4RnJvbVdhc20wKHIwLCByMSkuc2xpY2UoKTtcbiAgICAgICAgICAgIHdhc20uX193YmluZGdlbl9mcmVlKHIwLCByMSAqIDEpO1xuICAgICAgICAgICAgcmV0dXJuIHYxO1xuICAgICAgICB9IGZpbmFsbHkge1xuICAgICAgICAgICAgd2FzbS5fX3diaW5kZ2VuX2FkZF90b19zdGFja19wb2ludGVyKDE2KTtcbiAgICAgICAgfVxuICAgIH1cbiAgICAvKipcbiAgICAqIEBwYXJhbSB7VWludDhBcnJheX0gbXNnXG4gICAgKiBAcmV0dXJucyB7VWludDhBcnJheX1cbiAgICAqL1xuICAgIHF1ZXJ5KG1zZykge1xuICAgICAgICB0cnkge1xuICAgICAgICAgICAgaWYgKHRoaXMucHRyID09IDApIHRocm93IG5ldyBFcnJvcignQXR0ZW1wdCB0byB1c2UgYSBtb3ZlZCB2YWx1ZScpO1xuICAgICAgICAgICAgY29uc3QgcmV0cHRyID0gd2FzbS5fX3diaW5kZ2VuX2FkZF90b19zdGFja19wb2ludGVyKC0xNik7XG4gICAgICAgICAgICBfYXNzZXJ0TnVtKHRoaXMucHRyKTtcbiAgICAgICAgICAgIHZhciBwdHIwID0gcGFzc0FycmF5OFRvV2FzbTAobXNnLCB3YXNtLl9fd2JpbmRnZW5fbWFsbG9jKTtcbiAgICAgICAgICAgIHZhciBsZW4wID0gV0FTTV9WRUNUT1JfTEVOO1xuICAgICAgICAgICAgd2FzbS5jb250cmFjdF9xdWVyeShyZXRwdHIsIHRoaXMucHRyLCBwdHIwLCBsZW4wKTtcbiAgICAgICAgICAgIHZhciByMCA9IGdldEludDMyTWVtb3J5MCgpW3JldHB0ciAvIDQgKyAwXTtcbiAgICAgICAgICAgIHZhciByMSA9IGdldEludDMyTWVtb3J5MCgpW3JldHB0ciAvIDQgKyAxXTtcbiAgICAgICAgICAgIHZhciB2MSA9IGdldEFycmF5VThGcm9tV2FzbTAocjAsIHIxKS5zbGljZSgpO1xuICAgICAgICAgICAgd2FzbS5fX3diaW5kZ2VuX2ZyZWUocjAsIHIxICogMSk7XG4gICAgICAgICAgICByZXR1cm4gdjE7XG4gICAgICAgIH0gZmluYWxseSB7XG4gICAgICAgICAgICB3YXNtLl9fd2JpbmRnZW5fYWRkX3RvX3N0YWNrX3BvaW50ZXIoMTYpO1xuICAgICAgICB9XG4gICAgfVxufVxuXG5hc3luYyBmdW5jdGlvbiBsb2FkKG1vZHVsZSwgaW1wb3J0cykge1xuICAgIGlmICh0eXBlb2YgUmVzcG9uc2UgPT09ICdmdW5jdGlvbicgJiYgbW9kdWxlIGluc3RhbmNlb2YgUmVzcG9uc2UpIHtcbiAgICAgICAgaWYgKHR5cGVvZiBXZWJBc3NlbWJseS5pbnN0YW50aWF0ZVN0cmVhbWluZyA9PT0gJ2Z1bmN0aW9uJykge1xuICAgICAgICAgICAgdHJ5IHtcbiAgICAgICAgICAgICAgICByZXR1cm4gYXdhaXQgV2ViQXNzZW1ibHkuaW5zdGFudGlhdGVTdHJlYW1pbmcobW9kdWxlLCBpbXBvcnRzKTtcblxuICAgICAgICAgICAgfSBjYXRjaCAoZSkge1xuICAgICAgICAgICAgICAgIGlmIChtb2R1bGUuaGVhZGVycy5nZXQoJ0NvbnRlbnQtVHlwZScpICE9ICdhcHBsaWNhdGlvbi93YXNtJykge1xuICAgICAgICAgICAgICAgICAgICBjb25zb2xlLndhcm4oXCJgV2ViQXNzZW1ibHkuaW5zdGFudGlhdGVTdHJlYW1pbmdgIGZhaWxlZCBiZWNhdXNlIHlvdXIgc2VydmVyIGRvZXMgbm90IHNlcnZlIHdhc20gd2l0aCBgYXBwbGljYXRpb24vd2FzbWAgTUlNRSB0eXBlLiBGYWxsaW5nIGJhY2sgdG8gYFdlYkFzc2VtYmx5Lmluc3RhbnRpYXRlYCB3aGljaCBpcyBzbG93ZXIuIE9yaWdpbmFsIGVycm9yOlxcblwiLCBlKTtcblxuICAgICAgICAgICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgICAgICAgICAgIHRocm93IGU7XG4gICAgICAgICAgICAgICAgfVxuICAgICAgICAgICAgfVxuICAgICAgICB9XG5cbiAgICAgICAgY29uc3QgYnl0ZXMgPSBhd2FpdCBtb2R1bGUuYXJyYXlCdWZmZXIoKTtcbiAgICAgICAgcmV0dXJuIGF3YWl0IFdlYkFzc2VtYmx5Lmluc3RhbnRpYXRlKGJ5dGVzLCBpbXBvcnRzKTtcblxuICAgIH0gZWxzZSB7XG4gICAgICAgIGNvbnN0IGluc3RhbmNlID0gYXdhaXQgV2ViQXNzZW1ibHkuaW5zdGFudGlhdGUobW9kdWxlLCBpbXBvcnRzKTtcblxuICAgICAgICBpZiAoaW5zdGFuY2UgaW5zdGFuY2VvZiBXZWJBc3NlbWJseS5JbnN0YW5jZSkge1xuICAgICAgICAgICAgcmV0dXJuIHsgaW5zdGFuY2UsIG1vZHVsZSB9O1xuXG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgICByZXR1cm4gaW5zdGFuY2U7XG4gICAgICAgIH1cbiAgICB9XG59XG5cbmFzeW5jIGZ1bmN0aW9uIGluaXQoaW5wdXQpIHtcbiAgICBpZiAodHlwZW9mIGlucHV0ID09PSAndW5kZWZpbmVkJykge1xuICAgICAgICBpbnB1dCA9IG5ldyBVUkwoJ3Jld2FyZHNfYmcud2FzbScsIGltcG9ydC5tZXRhLnVybCk7XG4gICAgfVxuICAgIGNvbnN0IGltcG9ydHMgPSB7fTtcbiAgICBpbXBvcnRzLndiZyA9IHt9O1xuICAgIGltcG9ydHMud2JnLl9fd2JpbmRnZW5fc3RyaW5nX25ldyA9IGZ1bmN0aW9uKGFyZzAsIGFyZzEpIHtcbiAgICAgICAgdmFyIHJldCA9IGdldFN0cmluZ0Zyb21XYXNtMChhcmcwLCBhcmcxKTtcbiAgICAgICAgcmV0dXJuIGFkZEhlYXBPYmplY3QocmV0KTtcbiAgICB9O1xuICAgIGltcG9ydHMud2JnLl9fd2JpbmRnZW5fdGhyb3cgPSBmdW5jdGlvbihhcmcwLCBhcmcxKSB7XG4gICAgICAgIHRocm93IG5ldyBFcnJvcihnZXRTdHJpbmdGcm9tV2FzbTAoYXJnMCwgYXJnMSkpO1xuICAgIH07XG4gICAgaW1wb3J0cy53YmcuX193YmluZGdlbl9yZXRocm93ID0gZnVuY3Rpb24oYXJnMCkge1xuICAgICAgICB0aHJvdyB0YWtlT2JqZWN0KGFyZzApO1xuICAgIH07XG5cbiAgICBpZiAodHlwZW9mIGlucHV0ID09PSAnc3RyaW5nJyB8fCAodHlwZW9mIFJlcXVlc3QgPT09ICdmdW5jdGlvbicgJiYgaW5wdXQgaW5zdGFuY2VvZiBSZXF1ZXN0KSB8fCAodHlwZW9mIFVSTCA9PT0gJ2Z1bmN0aW9uJyAmJiBpbnB1dCBpbnN0YW5jZW9mIFVSTCkpIHtcbiAgICAgICAgaW5wdXQgPSBmZXRjaChpbnB1dCk7XG4gICAgfVxuXG5cblxuICAgIGNvbnN0IHsgaW5zdGFuY2UsIG1vZHVsZSB9ID0gYXdhaXQgbG9hZChhd2FpdCBpbnB1dCwgaW1wb3J0cyk7XG5cbiAgICB3YXNtID0gaW5zdGFuY2UuZXhwb3J0cztcbiAgICBpbml0Ll9fd2JpbmRnZW5fd2FzbV9tb2R1bGUgPSBtb2R1bGU7XG5cbiAgICByZXR1cm4gd2FzbTtcbn1cblxuZXhwb3J0IGRlZmF1bHQgaW5pdDtcblxuIiwiLy8gSW1wb3J0c1xuaW1wb3J0IF9fX0NTU19MT0FERVJfQVBJX1NPVVJDRU1BUF9JTVBPUlRfX18gZnJvbSBcIi4uLy4uLy4uL25vZGVfbW9kdWxlcy9jc3MtbG9hZGVyL2Rpc3QvcnVudGltZS9jc3NXaXRoTWFwcGluZ1RvU3RyaW5nLmpzXCI7XG5pbXBvcnQgX19fQ1NTX0xPQURFUl9BUElfSU1QT1JUX19fIGZyb20gXCIuLi8uLi8uLi9ub2RlX21vZHVsZXMvY3NzLWxvYWRlci9kaXN0L3J1bnRpbWUvYXBpLmpzXCI7XG52YXIgX19fQ1NTX0xPQURFUl9FWFBPUlRfX18gPSBfX19DU1NfTE9BREVSX0FQSV9JTVBPUlRfX18oX19fQ1NTX0xPQURFUl9BUElfU09VUkNFTUFQX0lNUE9SVF9fXyk7XG4vLyBNb2R1bGVcbl9fX0NTU19MT0FERVJfRVhQT1JUX19fLnB1c2goW21vZHVsZS5pZCwgXCIqIHtcXG4gIGJveC1zaXppbmc6IGJvcmRlci1ib3g7XFxuICBtYXJnaW46IDA7XFxuICBwYWRkaW5nOiAwO1xcbiAgZm9udC1zaXplOiAxcmVtO1xcbiAgbGlzdC1zdHlsZTogbm9uZTtcXG59XFxuXFxuaHRtbCwgYm9keSB7XFxuICBoZWlnaHQ6ICAxMDAlO1xcbiAgbWFyZ2luOiAgMDtcXG4gIHBhZGRpbmc6IDA7XFxuICBmb250LWZhbWlseTogc2Fucy1zZXJpZjtcXG59XFxuXFxuYm9keSB7XFxuICBiYWNrZ3JvdW5kOiAjMjgyODI4O1xcbiAgY29sb3I6ICAgICAgI2ViZGJiMjtcXG4gIGRpc3BsYXk6ICAgIGdyaWQ7XFxuICBncmlkLXRlbXBsYXRlLWNvbHVtbnM6IDIwJSAyMCUgMjAlIDIwJSAyMCU7XFxuICBncmlkLXRlbXBsYXRlLXJvd3M6IDUwJSA1MCVcXG59XFxuXFxuQG1lZGlhIChvcmllbnRhdGlvbjogbGFuZHNjYXBlKSB7fVxcbkBtZWRpYSAob3JpZW50YXRpb246IHBvcnRyYWl0KSB7fVxcblxcbmgxIHtcXG4gIGZvbnQtd2VpZ2h0OiBib2xkO1xcbiAgbWFyZ2luOiAwO1xcbn1cXG5oMiB7XFxuICBmb250LXdlaWdodDogbm9ybWFsO1xcbiAgdGV4dC1hbGlnbjogcmlnaHQ7XFxuICBtYXJnaW4tYm90dG9tOiAxcmVtO1xcbn1cXG5cXG4ucGllLCAuaGlzdG9yeSB7XFxuICAvKmZsZXgtZ3JvdzogMTsqL1xcbiAgLypmbGV4LXNocmluazogMTsqL1xcbiAgLypmbGV4LWJhc2lzOiAxMDAlOyovXFxufVxcblxcbi5waWUge1xcbiAgdGV4dC1hbGlnbjogbGVmdDtcXG4gIGdyaWQtY29sdW1uLXN0YXJ0OiAxO1xcbiAgZ3JpZC1jb2x1bW4tZW5kOiAgIDM7XFxufVxcbi5waWUuc3RhY2tlZCB7XFxuICBncmlkLWNvbHVtbi1zdGFydDogMztcXG4gIGdyaWQtY29sdW1uLWVuZDogICA1O1xcbn1cXG4gIC5waWUgaDEge1xcbiAgICBtYXJnaW46IDFlbSAxZW0gMCAxZW07XFxuICB9XFxuICAucGllIGNhbnZhcyB7XFxuICB9XFxuXFxudGFibGUge1xcbiAgZ3JpZC1yb3ctc3RhcnQ6IDI7XFxuICBncmlkLWNvbHVtbi1zdGFydDogMTtcXG4gIGdyaWQtY29sdW1uLWVuZDogNTtcXG4gIGJhY2tncm91bmQ6ICMzMjMwMmY7XFxuICBib3JkZXItY29sbGFwc2U6IGNvbGxhcHNlO1xcbn1cXG5cXG50ZCB7XFxuICBwYWRkaW5nOiAwLjI1cmVtO1xcbiAgdGV4dC1hbGlnbjogY2VudGVyO1xcbn1cXG50ZC5jbGFpbWFibGU6aG92ZXIge1xcbiAgY29sb3I6IHdoaXRlICFpbXBvcnRhbnQ7XFxuICBjdXJzb3I6IHBvaW50ZXI7XFxuICB0ZXh0LWRlY29yYXRpb246IHVuZGVybGluZTtcXG59XFxudGhlYWQge1xcbiAgYmFja2dyb3VuZDogIzUwNDk0NTtcXG59XFxudGgge1xcbiAgcGFkZGluZzogMC41cmVtO1xcbn1cXG5cXG4uaGlzdG9yeSB7XFxuICBncmlkLXJvdy1zdGFydDogMTtcXG4gIGdyaWQtcm93LWVuZDogMztcXG4gIGdyaWQtY29sdW1uLXN0YXJ0OiA1O1xcbiAgcGFkZGluZzogMWVtO1xcbiAgYmFja2dyb3VuZDogIzNjMzgzNjtcXG4gIG92ZXJmbG93OiBhdXRvO1xcbn1cXG5cXG5oMSwgaDIsIG9sIHtcXG4gIHBhZGRpbmc6IDAgMC41ZW07XFxufVxcblxcbi8qLnNwYXJrbGluZSB7Ki9cXG4gIC8qcG9zaXRpb246IGFic29sdXRlOyovXFxuICAvKmxlZnQ6IDA7Ki9cXG4gIC8qcmlnaHQ6IDA7Ki9cXG4gIC8qYm90dG9tOiAwOyovXFxuICAvKmhlaWdodDogMjB2aDsqL1xcbi8qfSovXFxuXFxuY2VudGVyIHtcXG4gIGdyaWQtY29sdW1uLXN0YXJ0OiAxO1xcbiAgZ3JpZC1jb2x1bW4tZW5kOiA2O1xcbiAgZ3JpZC1yb3ctc3RhcnQ6IDE7XFxuICBncmlkLXJvdy1lbmQ6IDM7XFxuICBkaXNwbGF5OiBmbGV4O1xcbiAgYWxpZ24taXRlbXM6IGNlbnRlcjtcXG4gIGp1c3RpZnktY29udGVudDogY2VudGVyO1xcbiAgZm9udC1zaXplOiAzcmVtO1xcbn1cXG5cXG4uZmllbGQge1xcbiAgcGFkZGluZy1ib3R0b206IDFyZW07XFxuICBib3JkZXItYm90dG9tOiAxcHggc29saWQgcmdiYSgwLDAsMCwwLjUpO1xcbiAgbWFyZ2luLWJvdHRvbTogMXJlbTtcXG59XFxuLmZpZWxkIGxhYmVsIHtcXG4gIGZvbnQtd2VpZ2h0OiBib2xkO1xcbn1cXG4uZmllbGQgPiBkaXYge1xcbiAgdGV4dC1hbGlnbjogcmlnaHRcXG59XFxuXFxudGQubG9ja2VkIHtcXG4gIGRpc3BsYXk6IGZsZXg7XFxuICBqdXN0aWZ5LWNvbnRlbnQ6IHNwYWNlLWJldHdlZW47XFxuICBiYWNrZ3JvdW5kOiByZ2JhKDAsMCwwLDAuMilcXG59XFxuLmxvY2tlZCBidXR0b24ge1xcbiAgcGFkZGluZzogMCAxcmVtO1xcbn1cXG5cIiwgXCJcIix7XCJ2ZXJzaW9uXCI6MyxcInNvdXJjZXNcIjpbXCJ3ZWJwYWNrOi8vLi9kYXNoYm9hcmQvc3R5bGUuY3NzXCJdLFwibmFtZXNcIjpbXSxcIm1hcHBpbmdzXCI6XCJBQUFBO0VBQ0Usc0JBQXNCO0VBQ3RCLFNBQVM7RUFDVCxVQUFVO0VBQ1YsZUFBZTtFQUNmLGdCQUFnQjtBQUNsQjs7QUFFQTtFQUNFLGFBQWE7RUFDYixVQUFVO0VBQ1YsVUFBVTtFQUNWLHVCQUF1QjtBQUN6Qjs7QUFFQTtFQUNFLG1CQUFtQjtFQUNuQixtQkFBbUI7RUFDbkIsZ0JBQWdCO0VBQ2hCLDBDQUEwQztFQUMxQztBQUNGOztBQUVBLGlDQUFpQztBQUNqQyxnQ0FBZ0M7O0FBRWhDO0VBQ0UsaUJBQWlCO0VBQ2pCLFNBQVM7QUFDWDtBQUNBO0VBQ0UsbUJBQW1CO0VBQ25CLGlCQUFpQjtFQUNqQixtQkFBbUI7QUFDckI7O0FBRUE7RUFDRSxnQkFBZ0I7RUFDaEIsa0JBQWtCO0VBQ2xCLG9CQUFvQjtBQUN0Qjs7QUFFQTtFQUNFLGdCQUFnQjtFQUNoQixvQkFBb0I7RUFDcEIsb0JBQW9CO0FBQ3RCO0FBQ0E7RUFDRSxvQkFBb0I7RUFDcEIsb0JBQW9CO0FBQ3RCO0VBQ0U7SUFDRSxxQkFBcUI7RUFDdkI7RUFDQTtFQUNBOztBQUVGO0VBQ0UsaUJBQWlCO0VBQ2pCLG9CQUFvQjtFQUNwQixrQkFBa0I7RUFDbEIsbUJBQW1CO0VBQ25CLHlCQUF5QjtBQUMzQjs7QUFFQTtFQUNFLGdCQUFnQjtFQUNoQixrQkFBa0I7QUFDcEI7QUFDQTtFQUNFLHVCQUF1QjtFQUN2QixlQUFlO0VBQ2YsMEJBQTBCO0FBQzVCO0FBQ0E7RUFDRSxtQkFBbUI7QUFDckI7QUFDQTtFQUNFLGVBQWU7QUFDakI7O0FBRUE7RUFDRSxpQkFBaUI7RUFDakIsZUFBZTtFQUNmLG9CQUFvQjtFQUNwQixZQUFZO0VBQ1osbUJBQW1CO0VBQ25CLGNBQWM7QUFDaEI7O0FBRUE7RUFDRSxnQkFBZ0I7QUFDbEI7O0FBRUEsZUFBZTtFQUNiLHNCQUFzQjtFQUN0QixXQUFXO0VBQ1gsWUFBWTtFQUNaLGFBQWE7RUFDYixnQkFBZ0I7QUFDbEIsSUFBSTs7QUFFSjtFQUNFLG9CQUFvQjtFQUNwQixrQkFBa0I7RUFDbEIsaUJBQWlCO0VBQ2pCLGVBQWU7RUFDZixhQUFhO0VBQ2IsbUJBQW1CO0VBQ25CLHVCQUF1QjtFQUN2QixlQUFlO0FBQ2pCOztBQUVBO0VBQ0Usb0JBQW9CO0VBQ3BCLHdDQUF3QztFQUN4QyxtQkFBbUI7QUFDckI7QUFDQTtFQUNFLGlCQUFpQjtBQUNuQjtBQUNBO0VBQ0U7QUFDRjs7QUFFQTtFQUNFLGFBQWE7RUFDYiw4QkFBOEI7RUFDOUI7QUFDRjtBQUNBO0VBQ0UsZUFBZTtBQUNqQlwiLFwic291cmNlc0NvbnRlbnRcIjpbXCIqIHtcXG4gIGJveC1zaXppbmc6IGJvcmRlci1ib3g7XFxuICBtYXJnaW46IDA7XFxuICBwYWRkaW5nOiAwO1xcbiAgZm9udC1zaXplOiAxcmVtO1xcbiAgbGlzdC1zdHlsZTogbm9uZTtcXG59XFxuXFxuaHRtbCwgYm9keSB7XFxuICBoZWlnaHQ6ICAxMDAlO1xcbiAgbWFyZ2luOiAgMDtcXG4gIHBhZGRpbmc6IDA7XFxuICBmb250LWZhbWlseTogc2Fucy1zZXJpZjtcXG59XFxuXFxuYm9keSB7XFxuICBiYWNrZ3JvdW5kOiAjMjgyODI4O1xcbiAgY29sb3I6ICAgICAgI2ViZGJiMjtcXG4gIGRpc3BsYXk6ICAgIGdyaWQ7XFxuICBncmlkLXRlbXBsYXRlLWNvbHVtbnM6IDIwJSAyMCUgMjAlIDIwJSAyMCU7XFxuICBncmlkLXRlbXBsYXRlLXJvd3M6IDUwJSA1MCVcXG59XFxuXFxuQG1lZGlhIChvcmllbnRhdGlvbjogbGFuZHNjYXBlKSB7fVxcbkBtZWRpYSAob3JpZW50YXRpb246IHBvcnRyYWl0KSB7fVxcblxcbmgxIHtcXG4gIGZvbnQtd2VpZ2h0OiBib2xkO1xcbiAgbWFyZ2luOiAwO1xcbn1cXG5oMiB7XFxuICBmb250LXdlaWdodDogbm9ybWFsO1xcbiAgdGV4dC1hbGlnbjogcmlnaHQ7XFxuICBtYXJnaW4tYm90dG9tOiAxcmVtO1xcbn1cXG5cXG4ucGllLCAuaGlzdG9yeSB7XFxuICAvKmZsZXgtZ3JvdzogMTsqL1xcbiAgLypmbGV4LXNocmluazogMTsqL1xcbiAgLypmbGV4LWJhc2lzOiAxMDAlOyovXFxufVxcblxcbi5waWUge1xcbiAgdGV4dC1hbGlnbjogbGVmdDtcXG4gIGdyaWQtY29sdW1uLXN0YXJ0OiAxO1xcbiAgZ3JpZC1jb2x1bW4tZW5kOiAgIDM7XFxufVxcbi5waWUuc3RhY2tlZCB7XFxuICBncmlkLWNvbHVtbi1zdGFydDogMztcXG4gIGdyaWQtY29sdW1uLWVuZDogICA1O1xcbn1cXG4gIC5waWUgaDEge1xcbiAgICBtYXJnaW46IDFlbSAxZW0gMCAxZW07XFxuICB9XFxuICAucGllIGNhbnZhcyB7XFxuICB9XFxuXFxudGFibGUge1xcbiAgZ3JpZC1yb3ctc3RhcnQ6IDI7XFxuICBncmlkLWNvbHVtbi1zdGFydDogMTtcXG4gIGdyaWQtY29sdW1uLWVuZDogNTtcXG4gIGJhY2tncm91bmQ6ICMzMjMwMmY7XFxuICBib3JkZXItY29sbGFwc2U6IGNvbGxhcHNlO1xcbn1cXG5cXG50ZCB7XFxuICBwYWRkaW5nOiAwLjI1cmVtO1xcbiAgdGV4dC1hbGlnbjogY2VudGVyO1xcbn1cXG50ZC5jbGFpbWFibGU6aG92ZXIge1xcbiAgY29sb3I6IHdoaXRlICFpbXBvcnRhbnQ7XFxuICBjdXJzb3I6IHBvaW50ZXI7XFxuICB0ZXh0LWRlY29yYXRpb246IHVuZGVybGluZTtcXG59XFxudGhlYWQge1xcbiAgYmFja2dyb3VuZDogIzUwNDk0NTtcXG59XFxudGgge1xcbiAgcGFkZGluZzogMC41cmVtO1xcbn1cXG5cXG4uaGlzdG9yeSB7XFxuICBncmlkLXJvdy1zdGFydDogMTtcXG4gIGdyaWQtcm93LWVuZDogMztcXG4gIGdyaWQtY29sdW1uLXN0YXJ0OiA1O1xcbiAgcGFkZGluZzogMWVtO1xcbiAgYmFja2dyb3VuZDogIzNjMzgzNjtcXG4gIG92ZXJmbG93OiBhdXRvO1xcbn1cXG5cXG5oMSwgaDIsIG9sIHtcXG4gIHBhZGRpbmc6IDAgMC41ZW07XFxufVxcblxcbi8qLnNwYXJrbGluZSB7Ki9cXG4gIC8qcG9zaXRpb246IGFic29sdXRlOyovXFxuICAvKmxlZnQ6IDA7Ki9cXG4gIC8qcmlnaHQ6IDA7Ki9cXG4gIC8qYm90dG9tOiAwOyovXFxuICAvKmhlaWdodDogMjB2aDsqL1xcbi8qfSovXFxuXFxuY2VudGVyIHtcXG4gIGdyaWQtY29sdW1uLXN0YXJ0OiAxO1xcbiAgZ3JpZC1jb2x1bW4tZW5kOiA2O1xcbiAgZ3JpZC1yb3ctc3RhcnQ6IDE7XFxuICBncmlkLXJvdy1lbmQ6IDM7XFxuICBkaXNwbGF5OiBmbGV4O1xcbiAgYWxpZ24taXRlbXM6IGNlbnRlcjtcXG4gIGp1c3RpZnktY29udGVudDogY2VudGVyO1xcbiAgZm9udC1zaXplOiAzcmVtO1xcbn1cXG5cXG4uZmllbGQge1xcbiAgcGFkZGluZy1ib3R0b206IDFyZW07XFxuICBib3JkZXItYm90dG9tOiAxcHggc29saWQgcmdiYSgwLDAsMCwwLjUpO1xcbiAgbWFyZ2luLWJvdHRvbTogMXJlbTtcXG59XFxuLmZpZWxkIGxhYmVsIHtcXG4gIGZvbnQtd2VpZ2h0OiBib2xkO1xcbn1cXG4uZmllbGQgPiBkaXYge1xcbiAgdGV4dC1hbGlnbjogcmlnaHRcXG59XFxuXFxudGQubG9ja2VkIHtcXG4gIGRpc3BsYXk6IGZsZXg7XFxuICBqdXN0aWZ5LWNvbnRlbnQ6IHNwYWNlLWJldHdlZW47XFxuICBiYWNrZ3JvdW5kOiByZ2JhKDAsMCwwLDAuMilcXG59XFxuLmxvY2tlZCBidXR0b24ge1xcbiAgcGFkZGluZzogMCAxcmVtO1xcbn1cXG5cIl0sXCJzb3VyY2VSb290XCI6XCJcIn1dKTtcbi8vIEV4cG9ydHNcbmV4cG9ydCBkZWZhdWx0IF9fX0NTU19MT0FERVJfRVhQT1JUX19fO1xuIiwiXCJ1c2Ugc3RyaWN0XCI7XG5cbi8qXG4gIE1JVCBMaWNlbnNlIGh0dHA6Ly93d3cub3BlbnNvdXJjZS5vcmcvbGljZW5zZXMvbWl0LWxpY2Vuc2UucGhwXG4gIEF1dGhvciBUb2JpYXMgS29wcGVycyBAc29rcmFcbiovXG4vLyBjc3MgYmFzZSBjb2RlLCBpbmplY3RlZCBieSB0aGUgY3NzLWxvYWRlclxuLy8gZXNsaW50LWRpc2FibGUtbmV4dC1saW5lIGZ1bmMtbmFtZXNcbm1vZHVsZS5leHBvcnRzID0gZnVuY3Rpb24gKGNzc1dpdGhNYXBwaW5nVG9TdHJpbmcpIHtcbiAgdmFyIGxpc3QgPSBbXTsgLy8gcmV0dXJuIHRoZSBsaXN0IG9mIG1vZHVsZXMgYXMgY3NzIHN0cmluZ1xuXG4gIGxpc3QudG9TdHJpbmcgPSBmdW5jdGlvbiB0b1N0cmluZygpIHtcbiAgICByZXR1cm4gdGhpcy5tYXAoZnVuY3Rpb24gKGl0ZW0pIHtcbiAgICAgIHZhciBjb250ZW50ID0gY3NzV2l0aE1hcHBpbmdUb1N0cmluZyhpdGVtKTtcblxuICAgICAgaWYgKGl0ZW1bMl0pIHtcbiAgICAgICAgcmV0dXJuIFwiQG1lZGlhIFwiLmNvbmNhdChpdGVtWzJdLCBcIiB7XCIpLmNvbmNhdChjb250ZW50LCBcIn1cIik7XG4gICAgICB9XG5cbiAgICAgIHJldHVybiBjb250ZW50O1xuICAgIH0pLmpvaW4oXCJcIik7XG4gIH07IC8vIGltcG9ydCBhIGxpc3Qgb2YgbW9kdWxlcyBpbnRvIHRoZSBsaXN0XG4gIC8vIGVzbGludC1kaXNhYmxlLW5leHQtbGluZSBmdW5jLW5hbWVzXG5cblxuICBsaXN0LmkgPSBmdW5jdGlvbiAobW9kdWxlcywgbWVkaWFRdWVyeSwgZGVkdXBlKSB7XG4gICAgaWYgKHR5cGVvZiBtb2R1bGVzID09PSBcInN0cmluZ1wiKSB7XG4gICAgICAvLyBlc2xpbnQtZGlzYWJsZS1uZXh0LWxpbmUgbm8tcGFyYW0tcmVhc3NpZ25cbiAgICAgIG1vZHVsZXMgPSBbW251bGwsIG1vZHVsZXMsIFwiXCJdXTtcbiAgICB9XG5cbiAgICB2YXIgYWxyZWFkeUltcG9ydGVkTW9kdWxlcyA9IHt9O1xuXG4gICAgaWYgKGRlZHVwZSkge1xuICAgICAgZm9yICh2YXIgaSA9IDA7IGkgPCB0aGlzLmxlbmd0aDsgaSsrKSB7XG4gICAgICAgIC8vIGVzbGludC1kaXNhYmxlLW5leHQtbGluZSBwcmVmZXItZGVzdHJ1Y3R1cmluZ1xuICAgICAgICB2YXIgaWQgPSB0aGlzW2ldWzBdO1xuXG4gICAgICAgIGlmIChpZCAhPSBudWxsKSB7XG4gICAgICAgICAgYWxyZWFkeUltcG9ydGVkTW9kdWxlc1tpZF0gPSB0cnVlO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuXG4gICAgZm9yICh2YXIgX2kgPSAwOyBfaSA8IG1vZHVsZXMubGVuZ3RoOyBfaSsrKSB7XG4gICAgICB2YXIgaXRlbSA9IFtdLmNvbmNhdChtb2R1bGVzW19pXSk7XG5cbiAgICAgIGlmIChkZWR1cGUgJiYgYWxyZWFkeUltcG9ydGVkTW9kdWxlc1tpdGVtWzBdXSkge1xuICAgICAgICAvLyBlc2xpbnQtZGlzYWJsZS1uZXh0LWxpbmUgbm8tY29udGludWVcbiAgICAgICAgY29udGludWU7XG4gICAgICB9XG5cbiAgICAgIGlmIChtZWRpYVF1ZXJ5KSB7XG4gICAgICAgIGlmICghaXRlbVsyXSkge1xuICAgICAgICAgIGl0ZW1bMl0gPSBtZWRpYVF1ZXJ5O1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIGl0ZW1bMl0gPSBcIlwiLmNvbmNhdChtZWRpYVF1ZXJ5LCBcIiBhbmQgXCIpLmNvbmNhdChpdGVtWzJdKTtcbiAgICAgICAgfVxuICAgICAgfVxuXG4gICAgICBsaXN0LnB1c2goaXRlbSk7XG4gICAgfVxuICB9O1xuXG4gIHJldHVybiBsaXN0O1xufTsiLCJcInVzZSBzdHJpY3RcIjtcblxuZnVuY3Rpb24gX3NsaWNlZFRvQXJyYXkoYXJyLCBpKSB7IHJldHVybiBfYXJyYXlXaXRoSG9sZXMoYXJyKSB8fCBfaXRlcmFibGVUb0FycmF5TGltaXQoYXJyLCBpKSB8fCBfdW5zdXBwb3J0ZWRJdGVyYWJsZVRvQXJyYXkoYXJyLCBpKSB8fCBfbm9uSXRlcmFibGVSZXN0KCk7IH1cblxuZnVuY3Rpb24gX25vbkl0ZXJhYmxlUmVzdCgpIHsgdGhyb3cgbmV3IFR5cGVFcnJvcihcIkludmFsaWQgYXR0ZW1wdCB0byBkZXN0cnVjdHVyZSBub24taXRlcmFibGUgaW5zdGFuY2UuXFxuSW4gb3JkZXIgdG8gYmUgaXRlcmFibGUsIG5vbi1hcnJheSBvYmplY3RzIG11c3QgaGF2ZSBhIFtTeW1ib2wuaXRlcmF0b3JdKCkgbWV0aG9kLlwiKTsgfVxuXG5mdW5jdGlvbiBfdW5zdXBwb3J0ZWRJdGVyYWJsZVRvQXJyYXkobywgbWluTGVuKSB7IGlmICghbykgcmV0dXJuOyBpZiAodHlwZW9mIG8gPT09IFwic3RyaW5nXCIpIHJldHVybiBfYXJyYXlMaWtlVG9BcnJheShvLCBtaW5MZW4pOyB2YXIgbiA9IE9iamVjdC5wcm90b3R5cGUudG9TdHJpbmcuY2FsbChvKS5zbGljZSg4LCAtMSk7IGlmIChuID09PSBcIk9iamVjdFwiICYmIG8uY29uc3RydWN0b3IpIG4gPSBvLmNvbnN0cnVjdG9yLm5hbWU7IGlmIChuID09PSBcIk1hcFwiIHx8IG4gPT09IFwiU2V0XCIpIHJldHVybiBBcnJheS5mcm9tKG8pOyBpZiAobiA9PT0gXCJBcmd1bWVudHNcIiB8fCAvXig/OlVpfEkpbnQoPzo4fDE2fDMyKSg/OkNsYW1wZWQpP0FycmF5JC8udGVzdChuKSkgcmV0dXJuIF9hcnJheUxpa2VUb0FycmF5KG8sIG1pbkxlbik7IH1cblxuZnVuY3Rpb24gX2FycmF5TGlrZVRvQXJyYXkoYXJyLCBsZW4pIHsgaWYgKGxlbiA9PSBudWxsIHx8IGxlbiA+IGFyci5sZW5ndGgpIGxlbiA9IGFyci5sZW5ndGg7IGZvciAodmFyIGkgPSAwLCBhcnIyID0gbmV3IEFycmF5KGxlbik7IGkgPCBsZW47IGkrKykgeyBhcnIyW2ldID0gYXJyW2ldOyB9IHJldHVybiBhcnIyOyB9XG5cbmZ1bmN0aW9uIF9pdGVyYWJsZVRvQXJyYXlMaW1pdChhcnIsIGkpIHsgdmFyIF9pID0gYXJyICYmICh0eXBlb2YgU3ltYm9sICE9PSBcInVuZGVmaW5lZFwiICYmIGFycltTeW1ib2wuaXRlcmF0b3JdIHx8IGFycltcIkBAaXRlcmF0b3JcIl0pOyBpZiAoX2kgPT0gbnVsbCkgcmV0dXJuOyB2YXIgX2FyciA9IFtdOyB2YXIgX24gPSB0cnVlOyB2YXIgX2QgPSBmYWxzZTsgdmFyIF9zLCBfZTsgdHJ5IHsgZm9yIChfaSA9IF9pLmNhbGwoYXJyKTsgIShfbiA9IChfcyA9IF9pLm5leHQoKSkuZG9uZSk7IF9uID0gdHJ1ZSkgeyBfYXJyLnB1c2goX3MudmFsdWUpOyBpZiAoaSAmJiBfYXJyLmxlbmd0aCA9PT0gaSkgYnJlYWs7IH0gfSBjYXRjaCAoZXJyKSB7IF9kID0gdHJ1ZTsgX2UgPSBlcnI7IH0gZmluYWxseSB7IHRyeSB7IGlmICghX24gJiYgX2lbXCJyZXR1cm5cIl0gIT0gbnVsbCkgX2lbXCJyZXR1cm5cIl0oKTsgfSBmaW5hbGx5IHsgaWYgKF9kKSB0aHJvdyBfZTsgfSB9IHJldHVybiBfYXJyOyB9XG5cbmZ1bmN0aW9uIF9hcnJheVdpdGhIb2xlcyhhcnIpIHsgaWYgKEFycmF5LmlzQXJyYXkoYXJyKSkgcmV0dXJuIGFycjsgfVxuXG5tb2R1bGUuZXhwb3J0cyA9IGZ1bmN0aW9uIGNzc1dpdGhNYXBwaW5nVG9TdHJpbmcoaXRlbSkge1xuICB2YXIgX2l0ZW0gPSBfc2xpY2VkVG9BcnJheShpdGVtLCA0KSxcbiAgICAgIGNvbnRlbnQgPSBfaXRlbVsxXSxcbiAgICAgIGNzc01hcHBpbmcgPSBfaXRlbVszXTtcblxuICBpZiAoIWNzc01hcHBpbmcpIHtcbiAgICByZXR1cm4gY29udGVudDtcbiAgfVxuXG4gIGlmICh0eXBlb2YgYnRvYSA9PT0gXCJmdW5jdGlvblwiKSB7XG4gICAgLy8gZXNsaW50LWRpc2FibGUtbmV4dC1saW5lIG5vLXVuZGVmXG4gICAgdmFyIGJhc2U2NCA9IGJ0b2EodW5lc2NhcGUoZW5jb2RlVVJJQ29tcG9uZW50KEpTT04uc3RyaW5naWZ5KGNzc01hcHBpbmcpKSkpO1xuICAgIHZhciBkYXRhID0gXCJzb3VyY2VNYXBwaW5nVVJMPWRhdGE6YXBwbGljYXRpb24vanNvbjtjaGFyc2V0PXV0Zi04O2Jhc2U2NCxcIi5jb25jYXQoYmFzZTY0KTtcbiAgICB2YXIgc291cmNlTWFwcGluZyA9IFwiLyojIFwiLmNvbmNhdChkYXRhLCBcIiAqL1wiKTtcbiAgICB2YXIgc291cmNlVVJMcyA9IGNzc01hcHBpbmcuc291cmNlcy5tYXAoZnVuY3Rpb24gKHNvdXJjZSkge1xuICAgICAgcmV0dXJuIFwiLyojIHNvdXJjZVVSTD1cIi5jb25jYXQoY3NzTWFwcGluZy5zb3VyY2VSb290IHx8IFwiXCIpLmNvbmNhdChzb3VyY2UsIFwiICovXCIpO1xuICAgIH0pO1xuICAgIHJldHVybiBbY29udGVudF0uY29uY2F0KHNvdXJjZVVSTHMpLmNvbmNhdChbc291cmNlTWFwcGluZ10pLmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICByZXR1cm4gW2NvbnRlbnRdLmpvaW4oXCJcXG5cIik7XG59OyIsIlxuICAgICAgaW1wb3J0IEFQSSBmcm9tIFwiIS4uLy4uLy4uL25vZGVfbW9kdWxlcy9zdHlsZS1sb2FkZXIvZGlzdC9ydW50aW1lL2luamVjdFN0eWxlc0ludG9TdHlsZVRhZy5qc1wiO1xuICAgICAgaW1wb3J0IGRvbUFQSSBmcm9tIFwiIS4uLy4uLy4uL25vZGVfbW9kdWxlcy9zdHlsZS1sb2FkZXIvZGlzdC9ydW50aW1lL3N0eWxlRG9tQVBJLmpzXCI7XG4gICAgICBpbXBvcnQgaW5zZXJ0Rm4gZnJvbSBcIiEuLi8uLi8uLi9ub2RlX21vZHVsZXMvc3R5bGUtbG9hZGVyL2Rpc3QvcnVudGltZS9pbnNlcnRCeVNlbGVjdG9yLmpzXCI7XG4gICAgICBpbXBvcnQgc2V0QXR0cmlidXRlcyBmcm9tIFwiIS4uLy4uLy4uL25vZGVfbW9kdWxlcy9zdHlsZS1sb2FkZXIvZGlzdC9ydW50aW1lL3NldEF0dHJpYnV0ZXNXaXRob3V0QXR0cmlidXRlcy5qc1wiO1xuICAgICAgaW1wb3J0IGluc2VydFN0eWxlRWxlbWVudCBmcm9tIFwiIS4uLy4uLy4uL25vZGVfbW9kdWxlcy9zdHlsZS1sb2FkZXIvZGlzdC9ydW50aW1lL2luc2VydFN0eWxlRWxlbWVudC5qc1wiO1xuICAgICAgaW1wb3J0IHN0eWxlVGFnVHJhbnNmb3JtRm4gZnJvbSBcIiEuLi8uLi8uLi9ub2RlX21vZHVsZXMvc3R5bGUtbG9hZGVyL2Rpc3QvcnVudGltZS9zdHlsZVRhZ1RyYW5zZm9ybS5qc1wiO1xuICAgICAgaW1wb3J0IGNvbnRlbnQsICogYXMgbmFtZWRFeHBvcnQgZnJvbSBcIiEhLi4vLi4vLi4vbm9kZV9tb2R1bGVzL2Nzcy1sb2FkZXIvZGlzdC9janMuanMhLi9zdHlsZS5jc3NcIjtcbiAgICAgIFxuICAgICAgXG5cbnZhciBvcHRpb25zID0ge307XG5cbm9wdGlvbnMuc3R5bGVUYWdUcmFuc2Zvcm0gPSBzdHlsZVRhZ1RyYW5zZm9ybUZuO1xub3B0aW9ucy5zZXRBdHRyaWJ1dGVzID0gc2V0QXR0cmlidXRlcztcblxuICAgICAgb3B0aW9ucy5pbnNlcnQgPSBpbnNlcnRGbi5iaW5kKG51bGwsIFwiaGVhZFwiKTtcbiAgICBcbm9wdGlvbnMuZG9tQVBJID0gZG9tQVBJO1xub3B0aW9ucy5pbnNlcnRTdHlsZUVsZW1lbnQgPSBpbnNlcnRTdHlsZUVsZW1lbnQ7XG5cbnZhciB1cGRhdGUgPSBBUEkoY29udGVudCwgb3B0aW9ucyk7XG5cblxuXG5leHBvcnQgKiBmcm9tIFwiISEuLi8uLi8uLi9ub2RlX21vZHVsZXMvY3NzLWxvYWRlci9kaXN0L2Nqcy5qcyEuL3N0eWxlLmNzc1wiO1xuICAgICAgIGV4cG9ydCBkZWZhdWx0IGNvbnRlbnQgJiYgY29udGVudC5sb2NhbHMgPyBjb250ZW50LmxvY2FscyA6IHVuZGVmaW5lZDtcbiIsIlwidXNlIHN0cmljdFwiO1xuXG52YXIgc3R5bGVzSW5Eb20gPSBbXTtcblxuZnVuY3Rpb24gZ2V0SW5kZXhCeUlkZW50aWZpZXIoaWRlbnRpZmllcikge1xuICB2YXIgcmVzdWx0ID0gLTE7XG5cbiAgZm9yICh2YXIgaSA9IDA7IGkgPCBzdHlsZXNJbkRvbS5sZW5ndGg7IGkrKykge1xuICAgIGlmIChzdHlsZXNJbkRvbVtpXS5pZGVudGlmaWVyID09PSBpZGVudGlmaWVyKSB7XG4gICAgICByZXN1bHQgPSBpO1xuICAgICAgYnJlYWs7XG4gICAgfVxuICB9XG5cbiAgcmV0dXJuIHJlc3VsdDtcbn1cblxuZnVuY3Rpb24gbW9kdWxlc1RvRG9tKGxpc3QsIG9wdGlvbnMpIHtcbiAgdmFyIGlkQ291bnRNYXAgPSB7fTtcbiAgdmFyIGlkZW50aWZpZXJzID0gW107XG5cbiAgZm9yICh2YXIgaSA9IDA7IGkgPCBsaXN0Lmxlbmd0aDsgaSsrKSB7XG4gICAgdmFyIGl0ZW0gPSBsaXN0W2ldO1xuICAgIHZhciBpZCA9IG9wdGlvbnMuYmFzZSA/IGl0ZW1bMF0gKyBvcHRpb25zLmJhc2UgOiBpdGVtWzBdO1xuICAgIHZhciBjb3VudCA9IGlkQ291bnRNYXBbaWRdIHx8IDA7XG4gICAgdmFyIGlkZW50aWZpZXIgPSBcIlwiLmNvbmNhdChpZCwgXCIgXCIpLmNvbmNhdChjb3VudCk7XG4gICAgaWRDb3VudE1hcFtpZF0gPSBjb3VudCArIDE7XG4gICAgdmFyIGluZGV4ID0gZ2V0SW5kZXhCeUlkZW50aWZpZXIoaWRlbnRpZmllcik7XG4gICAgdmFyIG9iaiA9IHtcbiAgICAgIGNzczogaXRlbVsxXSxcbiAgICAgIG1lZGlhOiBpdGVtWzJdLFxuICAgICAgc291cmNlTWFwOiBpdGVtWzNdXG4gICAgfTtcblxuICAgIGlmIChpbmRleCAhPT0gLTEpIHtcbiAgICAgIHN0eWxlc0luRG9tW2luZGV4XS5yZWZlcmVuY2VzKys7XG4gICAgICBzdHlsZXNJbkRvbVtpbmRleF0udXBkYXRlcihvYmopO1xuICAgIH0gZWxzZSB7XG4gICAgICBzdHlsZXNJbkRvbS5wdXNoKHtcbiAgICAgICAgaWRlbnRpZmllcjogaWRlbnRpZmllcixcbiAgICAgICAgdXBkYXRlcjogYWRkU3R5bGUob2JqLCBvcHRpb25zKSxcbiAgICAgICAgcmVmZXJlbmNlczogMVxuICAgICAgfSk7XG4gICAgfVxuXG4gICAgaWRlbnRpZmllcnMucHVzaChpZGVudGlmaWVyKTtcbiAgfVxuXG4gIHJldHVybiBpZGVudGlmaWVycztcbn1cblxuZnVuY3Rpb24gYWRkU3R5bGUob2JqLCBvcHRpb25zKSB7XG4gIHZhciBhcGkgPSBvcHRpb25zLmRvbUFQSShvcHRpb25zKTtcbiAgYXBpLnVwZGF0ZShvYmopO1xuICByZXR1cm4gZnVuY3Rpb24gdXBkYXRlU3R5bGUobmV3T2JqKSB7XG4gICAgaWYgKG5ld09iaikge1xuICAgICAgaWYgKG5ld09iai5jc3MgPT09IG9iai5jc3MgJiYgbmV3T2JqLm1lZGlhID09PSBvYmoubWVkaWEgJiYgbmV3T2JqLnNvdXJjZU1hcCA9PT0gb2JqLnNvdXJjZU1hcCkge1xuICAgICAgICByZXR1cm47XG4gICAgICB9XG5cbiAgICAgIGFwaS51cGRhdGUob2JqID0gbmV3T2JqKTtcbiAgICB9IGVsc2Uge1xuICAgICAgYXBpLnJlbW92ZSgpO1xuICAgIH1cbiAgfTtcbn1cblxubW9kdWxlLmV4cG9ydHMgPSBmdW5jdGlvbiAobGlzdCwgb3B0aW9ucykge1xuICBvcHRpb25zID0gb3B0aW9ucyB8fCB7fTtcbiAgbGlzdCA9IGxpc3QgfHwgW107XG4gIHZhciBsYXN0SWRlbnRpZmllcnMgPSBtb2R1bGVzVG9Eb20obGlzdCwgb3B0aW9ucyk7XG4gIHJldHVybiBmdW5jdGlvbiB1cGRhdGUobmV3TGlzdCkge1xuICAgIG5ld0xpc3QgPSBuZXdMaXN0IHx8IFtdO1xuXG4gICAgZm9yICh2YXIgaSA9IDA7IGkgPCBsYXN0SWRlbnRpZmllcnMubGVuZ3RoOyBpKyspIHtcbiAgICAgIHZhciBpZGVudGlmaWVyID0gbGFzdElkZW50aWZpZXJzW2ldO1xuICAgICAgdmFyIGluZGV4ID0gZ2V0SW5kZXhCeUlkZW50aWZpZXIoaWRlbnRpZmllcik7XG4gICAgICBzdHlsZXNJbkRvbVtpbmRleF0ucmVmZXJlbmNlcy0tO1xuICAgIH1cblxuICAgIHZhciBuZXdMYXN0SWRlbnRpZmllcnMgPSBtb2R1bGVzVG9Eb20obmV3TGlzdCwgb3B0aW9ucyk7XG5cbiAgICBmb3IgKHZhciBfaSA9IDA7IF9pIDwgbGFzdElkZW50aWZpZXJzLmxlbmd0aDsgX2krKykge1xuICAgICAgdmFyIF9pZGVudGlmaWVyID0gbGFzdElkZW50aWZpZXJzW19pXTtcblxuICAgICAgdmFyIF9pbmRleCA9IGdldEluZGV4QnlJZGVudGlmaWVyKF9pZGVudGlmaWVyKTtcblxuICAgICAgaWYgKHN0eWxlc0luRG9tW19pbmRleF0ucmVmZXJlbmNlcyA9PT0gMCkge1xuICAgICAgICBzdHlsZXNJbkRvbVtfaW5kZXhdLnVwZGF0ZXIoKTtcblxuICAgICAgICBzdHlsZXNJbkRvbS5zcGxpY2UoX2luZGV4LCAxKTtcbiAgICAgIH1cbiAgICB9XG5cbiAgICBsYXN0SWRlbnRpZmllcnMgPSBuZXdMYXN0SWRlbnRpZmllcnM7XG4gIH07XG59OyIsIlwidXNlIHN0cmljdFwiO1xuXG52YXIgbWVtbyA9IHt9O1xuLyogaXN0YW5idWwgaWdub3JlIG5leHQgICovXG5cbmZ1bmN0aW9uIGdldFRhcmdldCh0YXJnZXQpIHtcbiAgaWYgKHR5cGVvZiBtZW1vW3RhcmdldF0gPT09IFwidW5kZWZpbmVkXCIpIHtcbiAgICB2YXIgc3R5bGVUYXJnZXQgPSBkb2N1bWVudC5xdWVyeVNlbGVjdG9yKHRhcmdldCk7IC8vIFNwZWNpYWwgY2FzZSB0byByZXR1cm4gaGVhZCBvZiBpZnJhbWUgaW5zdGVhZCBvZiBpZnJhbWUgaXRzZWxmXG5cbiAgICBpZiAod2luZG93LkhUTUxJRnJhbWVFbGVtZW50ICYmIHN0eWxlVGFyZ2V0IGluc3RhbmNlb2Ygd2luZG93LkhUTUxJRnJhbWVFbGVtZW50KSB7XG4gICAgICB0cnkge1xuICAgICAgICAvLyBUaGlzIHdpbGwgdGhyb3cgYW4gZXhjZXB0aW9uIGlmIGFjY2VzcyB0byBpZnJhbWUgaXMgYmxvY2tlZFxuICAgICAgICAvLyBkdWUgdG8gY3Jvc3Mtb3JpZ2luIHJlc3RyaWN0aW9uc1xuICAgICAgICBzdHlsZVRhcmdldCA9IHN0eWxlVGFyZ2V0LmNvbnRlbnREb2N1bWVudC5oZWFkO1xuICAgICAgfSBjYXRjaCAoZSkge1xuICAgICAgICAvLyBpc3RhbmJ1bCBpZ25vcmUgbmV4dFxuICAgICAgICBzdHlsZVRhcmdldCA9IG51bGw7XG4gICAgICB9XG4gICAgfVxuXG4gICAgbWVtb1t0YXJnZXRdID0gc3R5bGVUYXJnZXQ7XG4gIH1cblxuICByZXR1cm4gbWVtb1t0YXJnZXRdO1xufVxuLyogaXN0YW5idWwgaWdub3JlIG5leHQgICovXG5cblxuZnVuY3Rpb24gaW5zZXJ0QnlTZWxlY3RvcihpbnNlcnQsIHN0eWxlKSB7XG4gIHZhciB0YXJnZXQgPSBnZXRUYXJnZXQoaW5zZXJ0KTtcblxuICBpZiAoIXRhcmdldCkge1xuICAgIHRocm93IG5ldyBFcnJvcihcIkNvdWxkbid0IGZpbmQgYSBzdHlsZSB0YXJnZXQuIFRoaXMgcHJvYmFibHkgbWVhbnMgdGhhdCB0aGUgdmFsdWUgZm9yIHRoZSAnaW5zZXJ0JyBwYXJhbWV0ZXIgaXMgaW52YWxpZC5cIik7XG4gIH1cblxuICB0YXJnZXQuYXBwZW5kQ2hpbGQoc3R5bGUpO1xufVxuXG5tb2R1bGUuZXhwb3J0cyA9IGluc2VydEJ5U2VsZWN0b3I7IiwiXCJ1c2Ugc3RyaWN0XCI7XG5cbi8qIGlzdGFuYnVsIGlnbm9yZSBuZXh0ICAqL1xuZnVuY3Rpb24gaW5zZXJ0U3R5bGVFbGVtZW50KG9wdGlvbnMpIHtcbiAgdmFyIHN0eWxlID0gZG9jdW1lbnQuY3JlYXRlRWxlbWVudChcInN0eWxlXCIpO1xuICBvcHRpb25zLnNldEF0dHJpYnV0ZXMoc3R5bGUsIG9wdGlvbnMuYXR0cmlidXRlcyk7XG4gIG9wdGlvbnMuaW5zZXJ0KHN0eWxlKTtcbiAgcmV0dXJuIHN0eWxlO1xufVxuXG5tb2R1bGUuZXhwb3J0cyA9IGluc2VydFN0eWxlRWxlbWVudDsiLCJcInVzZSBzdHJpY3RcIjtcblxuLyogaXN0YW5idWwgaWdub3JlIG5leHQgICovXG5mdW5jdGlvbiBzZXRBdHRyaWJ1dGVzV2l0aG91dEF0dHJpYnV0ZXMoc3R5bGUpIHtcbiAgdmFyIG5vbmNlID0gdHlwZW9mIF9fd2VicGFja19ub25jZV9fICE9PSBcInVuZGVmaW5lZFwiID8gX193ZWJwYWNrX25vbmNlX18gOiBudWxsO1xuXG4gIGlmIChub25jZSkge1xuICAgIHN0eWxlLnNldEF0dHJpYnV0ZShcIm5vbmNlXCIsIG5vbmNlKTtcbiAgfVxufVxuXG5tb2R1bGUuZXhwb3J0cyA9IHNldEF0dHJpYnV0ZXNXaXRob3V0QXR0cmlidXRlczsiLCJcInVzZSBzdHJpY3RcIjtcblxuLyogaXN0YW5idWwgaWdub3JlIG5leHQgICovXG5mdW5jdGlvbiBhcHBseShzdHlsZSwgb3B0aW9ucywgb2JqKSB7XG4gIHZhciBjc3MgPSBvYmouY3NzO1xuICB2YXIgbWVkaWEgPSBvYmoubWVkaWE7XG4gIHZhciBzb3VyY2VNYXAgPSBvYmouc291cmNlTWFwO1xuXG4gIGlmIChtZWRpYSkge1xuICAgIHN0eWxlLnNldEF0dHJpYnV0ZShcIm1lZGlhXCIsIG1lZGlhKTtcbiAgfSBlbHNlIHtcbiAgICBzdHlsZS5yZW1vdmVBdHRyaWJ1dGUoXCJtZWRpYVwiKTtcbiAgfVxuXG4gIGlmIChzb3VyY2VNYXAgJiYgdHlwZW9mIGJ0b2EgIT09IFwidW5kZWZpbmVkXCIpIHtcbiAgICBjc3MgKz0gXCJcXG4vKiMgc291cmNlTWFwcGluZ1VSTD1kYXRhOmFwcGxpY2F0aW9uL2pzb247YmFzZTY0LFwiLmNvbmNhdChidG9hKHVuZXNjYXBlKGVuY29kZVVSSUNvbXBvbmVudChKU09OLnN0cmluZ2lmeShzb3VyY2VNYXApKSkpLCBcIiAqL1wiKTtcbiAgfSAvLyBGb3Igb2xkIElFXG5cbiAgLyogaXN0YW5idWwgaWdub3JlIGlmICAqL1xuXG5cbiAgb3B0aW9ucy5zdHlsZVRhZ1RyYW5zZm9ybShjc3MsIHN0eWxlKTtcbn1cblxuZnVuY3Rpb24gcmVtb3ZlU3R5bGVFbGVtZW50KHN0eWxlKSB7XG4gIC8vIGlzdGFuYnVsIGlnbm9yZSBpZlxuICBpZiAoc3R5bGUucGFyZW50Tm9kZSA9PT0gbnVsbCkge1xuICAgIHJldHVybiBmYWxzZTtcbiAgfVxuXG4gIHN0eWxlLnBhcmVudE5vZGUucmVtb3ZlQ2hpbGQoc3R5bGUpO1xufVxuLyogaXN0YW5idWwgaWdub3JlIG5leHQgICovXG5cblxuZnVuY3Rpb24gZG9tQVBJKG9wdGlvbnMpIHtcbiAgdmFyIHN0eWxlID0gb3B0aW9ucy5pbnNlcnRTdHlsZUVsZW1lbnQob3B0aW9ucyk7XG4gIHJldHVybiB7XG4gICAgdXBkYXRlOiBmdW5jdGlvbiB1cGRhdGUob2JqKSB7XG4gICAgICBhcHBseShzdHlsZSwgb3B0aW9ucywgb2JqKTtcbiAgICB9LFxuICAgIHJlbW92ZTogZnVuY3Rpb24gcmVtb3ZlKCkge1xuICAgICAgcmVtb3ZlU3R5bGVFbGVtZW50KHN0eWxlKTtcbiAgICB9XG4gIH07XG59XG5cbm1vZHVsZS5leHBvcnRzID0gZG9tQVBJOyIsIlwidXNlIHN0cmljdFwiO1xuXG4vKiBpc3RhbmJ1bCBpZ25vcmUgbmV4dCAgKi9cbmZ1bmN0aW9uIHN0eWxlVGFnVHJhbnNmb3JtKGNzcywgc3R5bGUpIHtcbiAgaWYgKHN0eWxlLnN0eWxlU2hlZXQpIHtcbiAgICBzdHlsZS5zdHlsZVNoZWV0LmNzc1RleHQgPSBjc3M7XG4gIH0gZWxzZSB7XG4gICAgd2hpbGUgKHN0eWxlLmZpcnN0Q2hpbGQpIHtcbiAgICAgIHN0eWxlLnJlbW92ZUNoaWxkKHN0eWxlLmZpcnN0Q2hpbGQpO1xuICAgIH1cblxuICAgIHN0eWxlLmFwcGVuZENoaWxkKGRvY3VtZW50LmNyZWF0ZVRleHROb2RlKGNzcykpO1xuICB9XG59XG5cbm1vZHVsZS5leHBvcnRzID0gc3R5bGVUYWdUcmFuc2Zvcm07IiwiaW1wb3J0IHsgVUlDb250ZXh0IH0gZnJvbSAnLi93aWRnZXRzJ1xuaW1wb3J0IHsgQ09MT1JTIH0gZnJvbSAnLi9ncnV2Ym94J1xuXG4vLyBzZXR0aW5ncyAtLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tXG5leHBvcnQgY29uc3QgVElNRV9TQ0FMRSAgICAgICAgICA9IDEyMFxuICAgICAgICAgICAsIEZVTkRfUE9SVElPTlMgICAgICAgPSA3XG4gICAgICAgICAgICwgRElHSVRTICAgICAgICAgICAgICA9IDEwMDAwMDBcbiAgICAgICAgICAgLCBESUdJVFNfSU5WICAgICAgICAgID0gTWF0aC5sb2cxMChESUdJVFMpXG4gICAgICAgICAgICwgRlVORF9QT1JUSU9OICAgICAgICA9IDI1MDAgKiBESUdJVFNcbiAgICAgICAgICAgLCBGVU5EX0lOVEVSVkFMICAgICAgID0gMTcyODAvVElNRV9TQ0FMRVxuICAgICAgICAgICAsIENPT0xET1dOICAgICAgICAgICAgPSBGVU5EX0lOVEVSVkFMXG4gICAgICAgICAgICwgVEhSRVNIT0xEICAgICAgICAgICA9IEZVTkRfSU5URVJWQUxcbiAgICAgICAgICAgLCBVU0VSX0dJVkVTX1VQX0FGVEVSID0gSW5maW5pdHlcbiAgICAgICAgICAgLCBNQVhfVVNFUlMgICAgICAgICAgID0gMjBcbiAgICAgICAgICAgLCBNQVhfSU5JVElBTCAgICAgICAgID0gMTAwMDBcblxuZXhwb3J0IGNvbnN0IGZvcm1hdCA9IHtcbiAgaW50ZWdlcjogICAgKHg6bnVtYmVyKSA9PiBTdHJpbmcoeCksXG4gIGRlY2ltYWw6ICAgICh4Om51bWJlcikgPT4gKHgvRElHSVRTKS50b0ZpeGVkKERJR0lUU19JTlYpLFxuICBwZXJjZW50YWdlOiAoeDpudW1iZXIpID0+IGAke2Zvcm1hdC5kZWNpbWFsKHgpfSVgXG59XG5cbi8vIHJvb3Qgb2YgdGltZSAod2FybmluZywgc2luZ2xldG9uISkgLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS1cbmV4cG9ydCBjb25zdCBUID0geyBUOiAwIH1cblxuY2xhc3MgUlBUIHtcbiAgaW50ZXJ2YWwgID0gRlVORF9JTlRFUlZBTFxuICBwb3J0aW9uICAgPSBGVU5EX1BPUlRJT05cbiAgcmVtYWluaW5nID0gRlVORF9QT1JUSU9OU1xuICB2ZXN0ICgpIHtcbiAgICBpZiAoVC5UICUgdGhpcy5pbnRlcnZhbCA9PSAwKSB7XG4gICAgICBjb25zb2xlLmluZm8oJ2Z1bmQnLCB0aGlzLnBvcnRpb24sIHRoaXMucmVtYWluaW5nKVxuICAgICAgaWYgKHRoaXMucmVtYWluaW5nID4gMCkge1xuICAgICAgICB0aGlzLnBvcnRpb25cbiAgICAgICAgdGhpcy5yZW1haW5pbmcgLT0gMVxuICAgICAgICByZXR1cm4gdGhpcy5wb3J0aW9uXG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiAwXG4gIH1cbn1cblxuZXhwb3J0IGNsYXNzIFBvb2wge1xuICBycHQgPSBuZXcgUlBUKClcblxuICB1aTogICAgICAgICAgVUlDb250ZXh0XG4gIGxhc3RfdXBkYXRlOiBudW1iZXIgPSAwXG4gIGxpZmV0aW1lOiAgICBudW1iZXIgPSAwXG4gIGxvY2tlZDogICAgICBudW1iZXIgPSAwXG4gIGJhbGFuY2U6ICAgICBudW1iZXIgPSB0aGlzLnJwdC52ZXN0KClcbiAgY2xhaW1lZDogICAgIG51bWJlciA9IDBcbiAgY29vbGRvd246ICAgIG51bWJlciA9IDBcbiAgdGhyZXNob2xkOiAgIG51bWJlciA9IDBcbiAgbGlxdWlkOiAgICAgIG51bWJlciA9IDBcblxuICBjb25zdHJ1Y3RvciAodWk6IFVJQ29udGV4dCkge1xuICAgIHRoaXMudWkgPSB1aVxuICB9XG4gIHVwZGF0ZSAoKSB7XG4gICAgdGhpcy5iYWxhbmNlICs9IHRoaXMucnB0LnZlc3QoKVxuICAgIHRoaXMudWkubG9nLm5vdy5zZXRWYWx1ZShULlQpXG5cbiAgICB0aGlzLnVpLmxvZy5saWZldGltZS5zZXRWYWx1ZSh0aGlzLmxpZmV0aW1lKVxuICAgIHRoaXMudWkubG9nLmxvY2tlZC5zZXRWYWx1ZSh0aGlzLmxvY2tlZClcblxuICAgIHRoaXMudWkubG9nLmJhbGFuY2Uuc2V0VmFsdWUoZm9ybWF0LmRlY2ltYWwodGhpcy5iYWxhbmNlKSlcbiAgICB0aGlzLnVpLmxvZy5jbGFpbWVkLnNldFZhbHVlKGZvcm1hdC5kZWNpbWFsKHRoaXMuY2xhaW1lZCkpXG4gICAgdGhpcy51aS5sb2cucmVtYWluaW5nLnNldFZhbHVlKHRoaXMucnB0LnJlbWFpbmluZylcblxuICAgIHRoaXMudWkubG9nLmNvb2xkb3duLnNldFZhbHVlKHRoaXMuY29vbGRvd24pXG4gICAgdGhpcy51aS5sb2cudGhyZXNob2xkLnNldFZhbHVlKHRoaXMudGhyZXNob2xkKVxuICAgIHRoaXMudWkubG9nLmxpcXVpZC5zZXRWYWx1ZShmb3JtYXQucGVyY2VudGFnZSh0aGlzLmxpcXVpZCkpXG4gIH1cbn1cblxuZXhwb3J0IGNsYXNzIFVzZXIge1xuICB1aTogICAgICAgICAgIFVJQ29udGV4dFxuICBwb29sOiAgICAgICAgIFBvb2xcbiAgbmFtZTogICAgICAgICBzdHJpbmdcbiAgYmFsYW5jZTogICAgICBudW1iZXJcbiAgbGFzdF91cGRhdGU6ICBudW1iZXIgPSAwXG4gIGxpZmV0aW1lOiAgICAgbnVtYmVyID0gMFxuICBsb2NrZWQ6ICAgICAgIG51bWJlciA9IDBcbiAgYWdlOiAgICAgICAgICBudW1iZXIgPSAwXG4gIGVhcm5lZDogICAgICAgbnVtYmVyID0gMFxuICBjbGFpbWVkOiAgICAgIG51bWJlciA9IDBcbiAgY2xhaW1hYmxlOiAgICBudW1iZXIgPSAwXG4gIGNvb2xkb3duOiAgICAgbnVtYmVyID0gMFxuICB3YWl0ZWQ6ICAgICAgIG51bWJlciA9IDBcbiAgbGFzdF9jbGFpbWVkOiBudW1iZXIgPSAwXG4gIHNoYXJlOiAgICAgICAgbnVtYmVyID0gMFxuICBjb25zdHJ1Y3RvciAodWk6IFVJQ29udGV4dCwgcG9vbDogUG9vbCwgbmFtZTogc3RyaW5nLCBiYWxhbmNlOiBudW1iZXIpIHtcbiAgICB0aGlzLnVpICAgICAgPSB1aVxuICAgIHRoaXMucG9vbCAgICA9IHBvb2xcbiAgICB0aGlzLm5hbWUgICAgPSBuYW1lXG4gICAgdGhpcy5iYWxhbmNlID0gYmFsYW5jZVxuICB9XG4gIHVwZGF0ZSAoKSB7XG4gICAgdGhpcy51aS50YWJsZS51cGRhdGUodGhpcylcbiAgfVxuICBsb2NrIChhbW91bnQ6IG51bWJlcikge1xuICAgIHRoaXMudWkubG9nLmFkZCgnbG9ja3MnLCB0aGlzLm5hbWUsIGFtb3VudClcbiAgICB0aGlzLnVpLmN1cnJlbnQuYWRkKHRoaXMpXG4gICAgdGhpcy51aS5zdGFja2VkLmFkZCh0aGlzKVxuICB9XG4gIHJldHJpZXZlIChhbW91bnQ6IG51bWJlcikge1xuICAgIHRoaXMudWkubG9nLmFkZCgncmV0cmlldmVzJywgdGhpcy5uYW1lLCBhbW91bnQpXG4gICAgaWYgKHRoaXMubG9ja2VkID09PSAwKSB0aGlzLnVpLmN1cnJlbnQucmVtb3ZlKHRoaXMpXG4gIH1cbiAgY2xhaW0gKCkge1xuICAgIHRocm93IG5ldyBFcnJvcignbm90IGltcGxlbWVudGVkJylcbiAgfVxuICBkb0NsYWltIChyZXdhcmQ6IG51bWJlcikgeyAvLyBzdHVwaWQgdHlwZXNjcmlwdCBpbmhlcml0YW5jZSBjb25zdHJhaW50c1xuICAgIGNvbnNvbGUuZGVidWcodGhpcy5uYW1lLCAnY2xhaW0nLCByZXdhcmQpXG4gICAgaWYgKHJld2FyZCA8PSAwKSByZXR1cm4gMFxuXG4gICAgaWYgKHRoaXMubG9ja2VkID09PSAwKSByZXR1cm4gMFxuXG4gICAgaWYgKHRoaXMuY29vbGRvd24gPiAwIHx8IHRoaXMuYWdlIDwgVEhSRVNIT0xEKSByZXR1cm4gMFxuXG4gICAgaWYgKHRoaXMuY2xhaW1lZCA+IHRoaXMuZWFybmVkKSB7XG4gICAgICB0aGlzLnVpLmxvZy5hZGQoJ2Nyb3dkZWQgb3V0IEEnLCB0aGlzLm5hbWUsIHVuZGVmaW5lZClcbiAgICAgIHJldHVybiAwXG4gICAgfVxuXG4gICAgaWYgKHJld2FyZCA+IHRoaXMucG9vbC5iYWxhbmNlKSB7XG4gICAgICB0aGlzLnVpLmxvZy5hZGQoJ2Nyb3dkZWQgb3V0IEInLCB0aGlzLm5hbWUsIHVuZGVmaW5lZClcbiAgICAgIHJldHVybiAwXG4gICAgfVxuXG4gICAgdGhpcy5wb29sLmJhbGFuY2UgLT0gcmV3YXJkXG4gICAgdGhpcy51aS5sb2cuYWRkKCdjbGFpbScsIHRoaXMubmFtZSwgcmV3YXJkKVxuICAgIGNvbnNvbGUuZGVidWcoJ2NsYWltZWQ6JywgcmV3YXJkKVxuICAgIHJldHVybiByZXdhcmRcbiAgfVxuXG4gIGNvbG9ycyAoKSB7XG4gICAgcmV0dXJuIENPTE9SUyh0aGlzLnBvb2wsIHRoaXMpXG4gIH1cbn1cblxuZXhwb3J0IHR5cGUgVXNlcnMgPSBSZWNvcmQ8c3RyaW5nLCBVc2VyPlxuIiwiaW1wb3J0IHsgZW5jb2RlLCBkZWNvZGUgfSBmcm9tICcuL2hlbHBlcnMnXG5pbXBvcnQgeyBVSUNvbnRleHQgfSBmcm9tICcuL3dpZGdldHMnXG5pbXBvcnQgeyBULCBVc2VyLCBQb29sLCBUSFJFU0hPTEQsIENPT0xET1dOIH0gZnJvbSAnLi9jb250cmFjdF9iYXNlJ1xuaW1wb3J0IGluaXRSZXdhcmRzLCAqIGFzIEJvdW5kIGZyb20gJy4uL3RhcmdldC93ZWIvcmV3YXJkcy5qcydcblxuLy8gd3JhcHBlciBjbGFzc2VzIG9uIHRoZSBqcyBzaWRlIHRvby4uLiAtLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLVxuaW50ZXJmYWNlIExvZ0F0dHJpYnV0ZSB7XG4gIGtleTogICBzdHJpbmcsXG4gIHZhbHVlOiBzdHJpbmdcbn1cbmludGVyZmFjZSBIYW5kbGVSZXNwb25zZSB7XG4gIG1lc3NhZ2VzOiBBcnJheTxvYmplY3Q+LFxuICBsb2c6ICAgICAgYW55LFxuICBkYXRhOiAgICAgYW55XG59XG5jbGFzcyBSZXdhcmRzIHtcbiAgaW5kZXggPSAwXG4gIGNvbnRyYWN0ID0gbmV3IEJvdW5kLkNvbnRyYWN0KClcbiAgZGVidWcgPSBmYWxzZVxuICBpbml0IChtc2c6IG9iamVjdCkge1xuICAgIHRoaXMuaW5kZXggKz0gMVxuICAgIHRoaXMuYmxvY2sgPSBULlRcbiAgICAvL2lmICh0aGlzLmRlYnVnKSBjb25zb2xlLmRlYnVnKGBpbml0PiAke3RoaXMuaW5kZXh9YCwgbXNnKVxuICAgIGNvbnN0IHJlcyA9IGRlY29kZSh0aGlzLmNvbnRyYWN0LmluaXQoZW5jb2RlKG1zZykpKVxuICAgIC8vaWYgKHRoaXMuZGVidWcpIGNvbnNvbGUuZGVidWcoYDxpbml0ICR7dGhpcy5pbmRleH1gLCByZXMpXG4gICAgcmV0dXJuIHJlc1xuICB9XG4gIHF1ZXJ5IChtc2c6IG9iamVjdCkge1xuICAgIHRoaXMuaW5kZXggKz0gMVxuICAgIHRoaXMuYmxvY2sgPSBULlRcbiAgICAvL2lmICh0aGlzLmRlYnVnKSBjb25zb2xlLmRlYnVnKGBxdWVyeT4gJHt0aGlzLmluZGV4fWAsIG1zZylcbiAgICBjb25zdCByZXMgPSBkZWNvZGUodGhpcy5jb250cmFjdC5xdWVyeShlbmNvZGUobXNnKSkpXG4gICAgLy9pZiAodGhpcy5kZWJ1ZykgY29uc29sZS5kZWJ1ZyhgPHF1ZXJ5ICR7dGhpcy5pbmRleH1gLCByZXMpXG4gICAgcmV0dXJuIHJlc1xuICB9XG4gIGhhbmRsZSAobXNnOiBvYmplY3QpIHtcbiAgICB0aGlzLmluZGV4ICs9IDFcbiAgICB0aGlzLmJsb2NrID0gVC5UXG4gICAgLy9pZiAodGhpcy5kZWJ1ZykgY29uc29sZS5kZWJ1ZyhgaGFuZGxlPiAke3RoaXMuaW5kZXh9YCwgbXNnKVxuICAgIGNvbnN0IHJlczogSGFuZGxlUmVzcG9uc2UgPSBkZWNvZGUodGhpcy5jb250cmFjdC5oYW5kbGUoZW5jb2RlKG1zZykpKVxuICAgIHJlcy5sb2cgPSBPYmplY3QuZnJvbUVudHJpZXMoT2JqZWN0XG4gICAgICAudmFsdWVzKHJlcy5sb2cgYXMgb2JqZWN0KVxuICAgICAgLm1hcCgoe2tleSwgdmFsdWV9KT0+W2tleSwgdmFsdWVdKSlcbiAgICBpZiAoT2JqZWN0LmtleXMocmVzLmxvZykubGVuZ3RoID4gMCkgY29uc29sZS5sb2cocmVzLmxvZylcbiAgICAvL2lmICh0aGlzLmRlYnVnKSBjb25zb2xlLmRlYnVnKGA8aGFuZGxlICR7dGhpcy5pbmRleH1gLCByZXMpXG4gICAgcmV0dXJuIHJlc1xuICB9XG4gIHNldCBuZXh0X3F1ZXJ5X3Jlc3BvbnNlIChyZXNwb25zZTogb2JqZWN0KSB7XG4gICAgdGhpcy5jb250cmFjdC5uZXh0X3F1ZXJ5X3Jlc3BvbnNlID0gZW5jb2RlKHJlc3BvbnNlKVxuICB9XG4gIHNldCBzZW5kZXIgKGFkZHJlc3M6IHN0cmluZykge1xuICAgIHRoaXMuY29udHJhY3Quc2VuZGVyID0gZW5jb2RlKGFkZHJlc3MpXG4gIH1cbiAgc2V0IGJsb2NrIChoZWlnaHQ6IG51bWJlcikge1xuICAgIHRoaXMuY29udHJhY3QuYmxvY2sgPSBCaWdJbnQoaGVpZ2h0KVxuICB9XG59XG5cbi8vIHdhc20gbW9kdWxlIGxvYWQgJiBpbml0IC0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS1cbmV4cG9ydCBkZWZhdWx0IGFzeW5jIGZ1bmN0aW9uIGluaXRSZWFsICgpIHtcbiAgLy8gdGhhbmtmdWxseSB3YXNtLXBhY2svd2FzbS1iaW5kZ2VuIGxlZnQgYW4gZXNjYXBlIGhhdGNoXG4gIC8vIGJlY2F1c2UgaWRrIHd0ZiBpcyBnb2luZyBvbiB3aXRoIHRoZSBkZWZhdWx0IGxvYWRpbmcgY29kZVxuICBjb25zdCB1cmwgPSBuZXcgVVJMKCdyZXdhcmRzX2JnLndhc20nLCBsb2NhdGlvbi5ocmVmKVxuICAgICAgLCByZXMgPSBhd2FpdCBmZXRjaCh1cmwudG9TdHJpbmcoKSlcbiAgICAgICwgYnVmID0gYXdhaXQgcmVzLmFycmF5QnVmZmVyKClcbiAgYXdhaXQgaW5pdFJld2FyZHMoYnVmKVxufVxuXG4vLyBwb29sIGFwaSAtLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tXG5leHBvcnQgY2xhc3MgUmVhbFBvb2wgZXh0ZW5kcyBQb29sIHtcbiAgY29udHJhY3Q6IFJld2FyZHMgPSBuZXcgUmV3YXJkcygpXG4gIGNvbnN0cnVjdG9yICh1aTogVUlDb250ZXh0KSB7XG4gICAgc3VwZXIodWkpXG4gICAgdGhpcy5jb250cmFjdC5pbml0KHtcbiAgICAgIHJld2FyZF90b2tlbjogeyBhZGRyZXNzOiBcIlwiLCBjb2RlX2hhc2g6IFwiXCIgfSxcbiAgICAgIGxwX3Rva2VuOiAgICAgeyBhZGRyZXNzOiBcIlwiLCBjb2RlX2hhc2g6IFwiXCIgfSxcbiAgICAgIHZpZXdpbmdfa2V5OiAgXCJcIixcbiAgICAgIHRocmVzaG9sZDogICAgVEhSRVNIT0xELFxuICAgICAgY29vbGRvd246ICAgICBDT09MRE9XTlxuICAgIH0pXG4gICAgdGhpcy51aS5sb2cuY2xvc2Uub25jbGljayA9IHRoaXMuY2xvc2UuYmluZCh0aGlzKVxuICB9XG4gIHVwZGF0ZSAoKSB7XG4gICAgdGhpcy5jb250cmFjdC5uZXh0X3F1ZXJ5X3Jlc3BvbnNlID0ge2JhbGFuY2U6e2Ftb3VudDpTdHJpbmcodGhpcy5iYWxhbmNlKX19XG4gICAgY29uc3QgaW5mbyA9IHRoaXMuY29udHJhY3QucXVlcnkoe3Bvb2xfaW5mbzp7YXQ6VC5UfX0pLnBvb2xfaW5mb1xuICAgIC8vY29uc29sZS5sb2coaW5mbylcbiAgICB0aGlzLmxhc3RfdXBkYXRlID0gaW5mby5wb29sX2xhc3RfdXBkYXRlXG4gICAgdGhpcy5saWZldGltZSAgICA9IGluZm8ucG9vbF9saWZldGltZVxuICAgIHRoaXMubG9ja2VkICAgICAgPSBpbmZvLnBvb2xfbG9ja2VkXG4gICAgdGhpcy5jbGFpbWVkICAgICA9IGluZm8ucG9vbF9jbGFpbWVkXG4gICAgdGhpcy50aHJlc2hvbGQgICA9IGluZm8ucG9vbF90aHJlc2hvbGRcbiAgICB0aGlzLmNvb2xkb3duICAgID0gaW5mby5wb29sX2Nvb2xkb3duXG4gICAgdGhpcy5saXF1aWQgICAgICA9IGluZm8ucG9vbF9saXF1aWRcbiAgICBzdXBlci51cGRhdGUoKVxuICB9XG4gIGNsb3NlICgpIHtcbiAgICB0aGlzLmNvbnRyYWN0LnNlbmRlciA9IFwiXCJcbiAgICB0aGlzLmNvbnRyYWN0LmhhbmRsZSh7Y2xvc2VfcG9vbDp7bWVzc2FnZTpcInBvb2wgY2xvc2VkXCJ9fSlcbiAgfVxufVxuXG4vLyB1c2VyIGFwaSAtLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tXG5leHBvcnQgY2xhc3MgUmVhbFVzZXIgZXh0ZW5kcyBVc2VyIHtcblxuICBhZGRyZXNzOiBzdHJpbmdcblxuICBnZXQgY29udHJhY3QgKCkge1xuICAgIHJldHVybiAodGhpcy5wb29sIGFzIFJlYWxQb29sKS5jb250cmFjdFxuICB9XG5cbiAgY29uc3RydWN0b3IgKHVpOiBVSUNvbnRleHQsIHBvb2w6IFBvb2wsIG5hbWU6IHN0cmluZywgYmFsYW5jZTogbnVtYmVyKSB7XG4gICAgc3VwZXIodWksIHBvb2wsIG5hbWUsIGJhbGFuY2UpXG4gICAgdGhpcy5hZGRyZXNzID0gdGhpcy5uYW1lXG4gICAgdGhpcy5jb250cmFjdC5zZW5kZXIgPSB0aGlzLmFkZHJlc3NcbiAgICB0aGlzLmNvbnRyYWN0LmhhbmRsZSh7IHNldF92aWV3aW5nX2tleTogeyBrZXk6IFwiXCIgfSB9KVxuICB9XG5cbiAgdXBkYXRlICgpIHtcbiAgICAvLyBtb2NrIHRoZSB1c2VyJ3MgYmFsYW5jZSAtIGFjdHVhbGx5IHN0b3JlZCBvbiB0aGlzIHNhbWUgb2JqZWN0XG4gICAgLy8gYmVjYXVzZSB3ZSBkb24ndCBoYXZlIGEgc25pcDIwIGNvbnRyYWN0IHRvIG1haW50YWluIGl0XG4gICAgdGhpcy5jb250cmFjdC5uZXh0X3F1ZXJ5X3Jlc3BvbnNlID0ge2JhbGFuY2U6e2Ftb3VudDpTdHJpbmcodGhpcy5wb29sLmJhbGFuY2UpfX1cblxuICAgIC8vIGdldCB0aGUgdXNlcidzIGluZm8gYXMgc3RvcmVkIGFuZCBjYWxjdWxhdGVkIGJ5IHRoZSByZXdhcmRzIGNvbnRyYWN0XG4gICAgLy8gcHJlc3VtaW5nIHRoZSBhYm92ZSBtb2NrIGJhbGFuY2VcbiAgICBjb25zdCBpbmZvID0gdGhpcy5jb250cmFjdC5xdWVyeSh7dXNlcl9pbmZvOiB7IGF0OiBULlQsIGFkZHJlc3M6IHRoaXMuYWRkcmVzcywga2V5OiBcIlwiIH19KS51c2VyX2luZm9cbiAgICB0aGlzLmxhc3RfdXBkYXRlID0gaW5mby51c2VyX2xhc3RfdXBkYXRlXG4gICAgdGhpcy5saWZldGltZSAgICA9IE51bWJlcihpbmZvLnVzZXJfbGlmZXRpbWUpXG4gICAgdGhpcy5sb2NrZWQgICAgICA9IE51bWJlcihpbmZvLnVzZXJfbG9ja2VkKVxuICAgIHRoaXMuc2hhcmUgICAgICAgPSBOdW1iZXIoaW5mby51c2VyX3NoYXJlKVxuICAgIHRoaXMuYWdlICAgICAgICAgPSBOdW1iZXIoaW5mby51c2VyX2FnZSlcbiAgICB0aGlzLmVhcm5lZCAgICAgID0gTnVtYmVyKGluZm8udXNlcl9lYXJuZWQpXG4gICAgdGhpcy5jbGFpbWVkICAgICA9IE51bWJlcihpbmZvLnVzZXJfY2xhaW1lZClcbiAgICB0aGlzLmNsYWltYWJsZSAgID0gTnVtYmVyKGluZm8udXNlcl9jbGFpbWFibGUpXG4gICAgdGhpcy5jb29sZG93biAgICA9IE51bWJlcihpbmZvLnVzZXJfY29vbGRvd24pXG4gICAgc3VwZXIudXBkYXRlKClcbiAgfVxuXG4gIGxvY2sgKGFtb3VudDogbnVtYmVyKSB7XG4gICAgdGhpcy5jb250cmFjdC5zZW5kZXIgPSB0aGlzLmFkZHJlc3NcbiAgICB0cnkge1xuICAgICAgLy9jb25zb2xlLmRlYnVnKCdsb2NrJywgYW1vdW50KVxuICAgICAgdGhpcy5jb250cmFjdC5oYW5kbGUoeyBsb2NrOiB7IGFtb3VudDogU3RyaW5nKGFtb3VudCkgfSB9KVxuICAgICAgc3VwZXIubG9jayhhbW91bnQpXG4gICAgfSBjYXRjaCAoZSkge1xuICAgICAgLy9jb25zb2xlLmVycm9yKGUpXG4gICAgfVxuICB9XG5cbiAgcmV0cmlldmUgKGFtb3VudDogbnVtYmVyKSB7XG4gICAgdGhpcy5jb250cmFjdC5zZW5kZXIgPSB0aGlzLmFkZHJlc3NcbiAgICB0cnkge1xuICAgICAgLy9jb25zb2xlLmRlYnVnKCdyZXRyaWV2ZScsIGFtb3VudClcbiAgICAgIHRoaXMuY29udHJhY3QuaGFuZGxlKHsgcmV0cmlldmU6IHsgYW1vdW50OiBTdHJpbmcoYW1vdW50KSB9IH0pXG4gICAgICBzdXBlci5yZXRyaWV2ZShhbW91bnQpXG4gICAgfSBjYXRjaCAoZSkge1xuICAgICAgLy9jb25zb2xlLmVycm9yKGUpXG4gICAgfVxuICB9XG5cbiAgY2xhaW0gKCkge1xuICAgIHRoaXMuY29udHJhY3Quc2VuZGVyID0gdGhpcy5hZGRyZXNzXG4gICAgdHJ5IHtcbiAgICAgIGNvbnN0IHJlc3VsdCA9IHRoaXMuY29udHJhY3QuaGFuZGxlKHsgY2xhaW06IHt9IH0pXG4gICAgICBjb25zdCByZXdhcmQgPSBOdW1iZXIocmVzdWx0LmxvZy5yZXdhcmQpXG4gICAgICByZXR1cm4gdGhpcy5kb0NsYWltKHJld2FyZClcbiAgICB9IGNhdGNoIChlKSB7XG4gICAgICBjb25zb2xlLmVycm9yKGUpXG4gICAgICByZXR1cm4gMFxuICAgIH1cbiAgfVxufVxuIiwiLy8gaHR0cHM6Ly9naXQuc25vb3QuY2x1Yi9jaGVlL2dydXZib3guanMvc3JjL2JyYW5jaC9tYXN0ZXIvTElDRU5TRVxuXG5jb25zdCBkYXJrMEhhcmQgPSAnIzFkMjAyMSdcbmNvbnN0IGRhcmswID0gJyMyODI4MjgnXG5jb25zdCBkYXJrMFNvZnQgPSAnIzMyMzAyZidcbmNvbnN0IGRhcmsxID0gJyMzYzM4MzYnXG5jb25zdCBkYXJrMiA9ICcjNTA0OTQ1J1xuY29uc3QgZGFyazMgPSAnIzY2NWM1NCdcbmNvbnN0IGRhcms0ID0gJyM3YzZmNjQnXG5cbmNvbnN0IGdyYXkyNDUgPSAnIzkyODM3NCdcbmNvbnN0IGdyYXkyNDQgPSAnIzkyODM3NCdcblxuY29uc3QgbGlnaHQwSGFyZCA9ICcjZjlmNWQ3J1xuY29uc3QgbGlnaHQwID0gJyNmYmYxYzcnXG5jb25zdCBsaWdodDBTb2Z0ID0gJyNmMmU1YmMnXG5jb25zdCBsaWdodDEgPSAnI2ViZGJiMidcbmNvbnN0IGxpZ2h0MiA9ICcjZDVjNGExJ1xuY29uc3QgbGlnaHQzID0gJyNiZGFlOTMnXG5jb25zdCBsaWdodDQgPSAnI2E4OTk4NCdcblxuY29uc3QgYnJpZ2h0UmVkID0gJyNmYjQ5MzQnXG5jb25zdCBicmlnaHRHcmVlbiA9ICcjYjhiYjI2J1xuY29uc3QgYnJpZ2h0WWVsbG93ID0gJyNmYWJkMmYnXG5jb25zdCBicmlnaHRCbHVlID0gJyM4M2E1OTgnXG5jb25zdCBicmlnaHRQdXJwbGUgPSAnI2QzODY5YidcbmNvbnN0IGJyaWdodEFxdWEgPSAnIzhlYzA3YydcbmNvbnN0IGJyaWdodE9yYW5nZSA9ICcjZmU4MDE5J1xuXG5jb25zdCBuZXV0cmFsUmVkID0gJyNjYzI0MWQnXG5jb25zdCBuZXV0cmFsR3JlZW4gPSAnIzk4OTcxYSdcbmNvbnN0IG5ldXRyYWxZZWxsb3cgPSAnI2Q3OTkyMSdcbmNvbnN0IG5ldXRyYWxCbHVlID0gJyM0NTg1ODgnXG5jb25zdCBuZXV0cmFsUHVycGxlID0gJyNiMTYyODYnXG5jb25zdCBuZXV0cmFsQXF1YSA9ICcjNjg5ZDZhJ1xuY29uc3QgbmV1dHJhbE9yYW5nZSA9ICcjZDY1ZDBlJ1xuXG5jb25zdCBmYWRlZFJlZCA9ICcjOWQwMDA2J1xuY29uc3QgZmFkZWRHcmVlbiA9ICcjNzk3NDBlJ1xuY29uc3QgZmFkZWRZZWxsb3cgPSAnI2I1NzYxNCdcbmNvbnN0IGZhZGVkQmx1ZSA9ICcjMDc2Njc4J1xuY29uc3QgZmFkZWRQdXJwbGUgPSAnIzhmM2Y3MSdcbmNvbnN0IGZhZGVkQXF1YSA9ICcjNDI3YjU4J1xuY29uc3QgZmFkZWRPcmFuZ2UgPSAnI2FmM2EwMydcblxuY29uc3QgR3J1dmJveCA9IHtcbiAgZGFyazBIYXJkLFxuICBkYXJrMFNvZnQsXG4gIGRhcmswLFxuICBkYXJrMSxcbiAgZGFyazIsXG4gIGRhcmszLFxuICBkYXJrNCxcbiAgZGFyazoge1xuICAgIGhhcmQ6IGRhcmswSGFyZCxcbiAgICBzb2Z0OiBkYXJrMFNvZnQsXG4gICAgMDogZGFyazAsXG4gICAgMTogZGFyazEsXG4gICAgMjogZGFyazIsXG4gICAgMzogZGFyazMsXG4gICAgNDogZGFyazRcbiAgfSxcblxuICBncmF5MjQ0LFxuICBncmF5MjQ1LFxuICBncmF5OiB7XG4gICAgMjQ0OiBncmF5MjQ0LFxuICAgIDI0NTogZ3JheTI0NVxuICB9LFxuXG4gIGxpZ2h0MEhhcmQsXG4gIGxpZ2h0MFNvZnQsXG4gIGxpZ2h0MCxcbiAgbGlnaHQxLFxuICBsaWdodDIsXG4gIGxpZ2h0MyxcbiAgbGlnaHQ0LFxuICBsaWdodDoge1xuICAgIGhhcmQ6IGxpZ2h0MEhhcmQsXG4gICAgc29mdDogbGlnaHQwU29mdCxcbiAgICAwOiBsaWdodDAsXG4gICAgMTogbGlnaHQxLFxuICAgIDI6IGxpZ2h0MixcbiAgICAzOiBsaWdodDMsXG4gICAgNDogbGlnaHQ0XG4gIH0sXG5cbiAgYnJpZ2h0UmVkLFxuICBicmlnaHRHcmVlbixcbiAgYnJpZ2h0WWVsbG93LFxuICBicmlnaHRCbHVlLFxuICBicmlnaHRQdXJwbGUsXG4gIGJyaWdodEFxdWEsXG4gIGJyaWdodE9yYW5nZSxcbiAgYnJpZ2h0OiB7XG4gICAgcmVkOiBicmlnaHRSZWQsXG4gICAgZ3JlZW46IGJyaWdodEdyZWVuLFxuICAgIHllbGxvdzogYnJpZ2h0WWVsbG93LFxuICAgIGJsdWU6IGJyaWdodEJsdWUsXG4gICAgcHVycGxlOiBicmlnaHRQdXJwbGUsXG4gICAgYXF1YTogYnJpZ2h0QXF1YSxcbiAgICBvcmFuZ2U6IGJyaWdodE9yYW5nZVxuICB9LFxuXG4gIG5ldXRyYWxSZWQsXG4gIG5ldXRyYWxHcmVlbixcbiAgbmV1dHJhbFllbGxvdyxcbiAgbmV1dHJhbEJsdWUsXG4gIG5ldXRyYWxQdXJwbGUsXG4gIG5ldXRyYWxBcXVhLFxuICBuZXV0cmFsT3JhbmdlLFxuICBuZXV0cmFsOiB7XG4gICAgcmVkOiBuZXV0cmFsUmVkLFxuICAgIGdyZWVuOiBuZXV0cmFsR3JlZW4sXG4gICAgeWVsbG93OiBuZXV0cmFsWWVsbG93LFxuICAgIGJsdWU6IG5ldXRyYWxCbHVlLFxuICAgIHB1cnBsZTogbmV1dHJhbFB1cnBsZSxcbiAgICBhcXVhOiBuZXV0cmFsQXF1YSxcbiAgICBvcmFuZ2U6IG5ldXRyYWxPcmFuZ2VcbiAgfSxcblxuICBmYWRlZFJlZCxcbiAgZmFkZWRHcmVlbixcbiAgZmFkZWRZZWxsb3csXG4gIGZhZGVkQmx1ZSxcbiAgZmFkZWRQdXJwbGUsXG4gIGZhZGVkQXF1YSxcbiAgZmFkZWRPcmFuZ2UsXG4gIGZhZGVkOiB7XG4gICAgcmVkOiBmYWRlZFJlZCxcbiAgICBncmVlbjogZmFkZWRHcmVlbixcbiAgICB5ZWxsb3c6IGZhZGVkWWVsbG93LFxuICAgIGJsdWU6IGZhZGVkQmx1ZSxcbiAgICBwdXJwbGU6IGZhZGVkUHVycGxlLFxuICAgIGFxdWE6IGZhZGVkQXF1YSxcbiAgICBvcmFuZ2U6IGZhZGVkT3JhbmdlXG4gIH1cbn1cblxuZXhwb3J0IGRlZmF1bHQgR3J1dmJveFxuXG5pbXBvcnQgeyBQb29sLCBVc2VyLCBUSFJFU0hPTEQgfSBmcm9tICcuL2NvbnRyYWN0X2Jhc2UnXG5leHBvcnQgY29uc3QgQ09MT1JTID0gT2JqZWN0LmFzc2lnbihcbiAgZnVuY3Rpb24gZ2V0Q29sb3IgKHBvb2w6IFBvb2wsIHVzZXI6IFVzZXIpIHtcbiAgICBzd2l0Y2ggKHRydWUpIHtcbiAgICAgIGNhc2UgdXNlci5hZ2UgPCBUSFJFU0hPTEQgfHwgdXNlci5jb29sZG93biA+IDA6IC8vIHdhaXRpbmcgZm9yIGFnZSB0aHJlc2hvbGRcbiAgICAgICAgcmV0dXJuIENPTE9SUy5DT09MRE9XTlxuICAgICAgY2FzZSB1c2VyLmNsYWltYWJsZSA+IDAgJiYgdXNlci5jb29sZG93biA9PSAxOiAgLy8gaGF2ZSByZXdhcmRzIHRvIGNsYWltXG4gICAgICAgIHJldHVybiBDT0xPUlMuQ0xBSU1JTkdcbiAgICAgIC8vY2FzZSB1c2VyLmNsYWltYWJsZSA+IDAgJiYgdXNlci5jb29sZG93biA+IDA6IC8vIGp1c3QgY2xhaW1lZCwgY29vbGluZyBkb3duXG4gICAgICAgIC8vcmV0dXJuIENPTE9SUy5BTExfT0tcbiAgICAgIGNhc2UgdXNlci5jbGFpbWFibGUgPiBwb29sLmJhbGFuY2U6ICAgICAgICAgICAgIC8vIG5vdCBlbm91Z2ggbW9uZXkgaW4gcG9vbFxuICAgICAgICByZXR1cm4gQ09MT1JTLkJMT0NLRUQgXG4gICAgICBjYXNlIHVzZXIuY2xhaW1lZCA+IHVzZXIuZWFybmVkOiAgICAgICAgICAgICAgICAvLyBjcm93ZGVkIG91dFxuICAgICAgICByZXR1cm4gQ09MT1JTLkNST1dERURcbiAgICAgIGNhc2UgdXNlci5jbGFpbWFibGUgPT09IDA6XG4gICAgICAgIHJldHVybiBDT0xPUlMuTk9USElOR1xuICAgICAgZGVmYXVsdDpcbiAgICAgICAgcmV0dXJuIENPTE9SUy5DTEFJTUFCTEVcbiAgICB9XG4gIH0sIHtcbiAgICBDTEFJTUFCTEU6IFtHcnV2Ym94LmZhZGVkQXF1YSwgICBHcnV2Ym94LmJyaWdodEFxdWFdLFxuICAgIENMQUlNSU5HOiAgW0dydXZib3guYnJpZ2h0QXF1YSwgIEdydXZib3guYnJpZ2h0QXF1YV0sXG4gICAgQkxPQ0tFRDogICBbR3J1dmJveC5mYWRlZE9yYW5nZSwgR3J1dmJveC5icmlnaHRPcmFuZ2VdLFxuICAgIENST1dERUQ6ICAgW0dydXZib3guZmFkZWRQdXJwbGUsIEdydXZib3guYnJpZ2h0UHVycGxlXSxcbiAgICBDT09MRE9XTjogIFtHcnV2Ym94LmZhZGVkQmx1ZSwgICBHcnV2Ym94LmJyaWdodEJsdWVdLFxuICAgIE5PVEhJTkc6ICAgW0dydXZib3guZGFyazAsICAgICAgIEdydXZib3guYnJpZ2h0WWVsbG93XVxuICB9KVxuIiwiLy8gcmFuZG9tbmVzcyBoZWxwZXJzIC0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLVxuXG5leHBvcnQgY29uc3QgcmFuZG9tID0gKG1heDogbnVtYmVyKSA9PlxuICBNYXRoLmZsb29yKE1hdGgucmFuZG9tKCkqbWF4KVxuXG5leHBvcnQgY29uc3QgcGlja1JhbmRvbSA9ICh4OiBhbnkpID0+XG4gIHhbcmFuZG9tKHgubGVuZ3RoKV1cblxuLy8gdGltaW5nIGhlbHBlcnMgLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLVxuXG5leHBvcnQgZnVuY3Rpb24gdGhyb3R0bGUgKHQ6IG51bWJlciwgZm46IEZ1bmN0aW9uKSB7XG4gIC8vIHRvZG8gcmVwbGFjaW5nIHQgd2l0aCBhIGZ1bmN0aW9uIGFsbG93cyBmb3IgaW1wbGVtZW50aW5nIGV4cG9uZW50aWFsIGJhY2tvZmZcbiAgbGV0IHRpbWVvdXQ6IGFueVxuICByZXR1cm4gZnVuY3Rpb24gdGhyb3R0bGVkICguLi5hcmdzOmFueSkge1xuICAgIHJldHVybiBuZXcgUHJvbWlzZShyZXNvbHZlPT57XG4gICAgICBpZiAodGltZW91dCkgY2xlYXJUaW1lb3V0KHRpbWVvdXQpXG4gICAgICB0aW1lb3V0ID0gYWZ0ZXIodCwgKCk9PnJlc29sdmUoZm4oLi4uYXJncykpKSB9KX19XG5cbmV4cG9ydCBmdW5jdGlvbiBhZnRlciAodDogbnVtYmVyLCBmbjogRnVuY3Rpb24pIHtcbiAgcmV0dXJuIHNldFRpbWVvdXQoZm4sIHQpIH1cblxuLy8gRE9NIGhlbHBlcnMgLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLVxuXG5leHBvcnQgZnVuY3Rpb24gaCAoZWxlbWVudDogc3RyaW5nLCBhdHRyaWJ1dGVzPXt9LCAuLi5jb250ZW50OmFueSkge1xuICBjb25zdCBlbCA9IE9iamVjdC5hc3NpZ24oZG9jdW1lbnQuY3JlYXRlRWxlbWVudChlbGVtZW50KSwgYXR0cmlidXRlcylcbiAgZm9yIChjb25zdCBlbDIgb2YgY29udGVudCkgZWwuYXBwZW5kQ2hpbGQoZWwyKVxuICByZXR1cm4gZWwgfVxuXG5leHBvcnQgZnVuY3Rpb24gYXBwZW5kIChwYXJlbnQ6IEhUTUxFbGVtZW50LCBjaGlsZDogSFRNTEVsZW1lbnQpIHtcbiAgcmV0dXJuIHBhcmVudC5hcHBlbmRDaGlsZChjaGlsZCkgfVxuXG5leHBvcnQgZnVuY3Rpb24gcHJlcGVuZCAocGFyZW50OiBIVE1MRWxlbWVudCwgY2hpbGQ6IEhUTUxFbGVtZW50KSB7XG4gIHJldHVybiBwYXJlbnQuaW5zZXJ0QmVmb3JlKGNoaWxkLCBwYXJlbnQuZmlyc3RDaGlsZCkgfVxuXG4vLyBjb252ZXJ0IGZyb20gc3RyaW5nIHRvIFV0ZjhBcnJheSAtLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tXG5cbmNvbnN0IGVuYyA9IG5ldyBUZXh0RW5jb2RlcigpXG5leHBvcnQgY29uc3QgZW5jb2RlID0gKHg6IGFueSkgPT4gZW5jLmVuY29kZShKU09OLnN0cmluZ2lmeSh4KSlcblxuY29uc3QgZGVjID0gbmV3IFRleHREZWNvZGVyKClcbmV4cG9ydCBjb25zdCBkZWNvZGUgPSAoeDogVWludDhBcnJheSkgPT4gSlNPTi5wYXJzZShkZWMuZGVjb2RlKHguYnVmZmVyKSlcbiIsImltcG9ydCB7IGgsIGFwcGVuZCwgcHJlcGVuZCB9IGZyb20gJy4vaGVscGVycydcbmltcG9ydCB7IFQsIFVzZXIsIFVzZXJzLCBmb3JtYXQgfSBmcm9tICcuL2NvbnRyYWN0X2Jhc2UnXG5cbi8vIGtpbGxzd2l0Y2hlcyBmb3IgZ3VpIGNvbXBvbmVudHMgLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS1cbmV4cG9ydCBjb25zdCBOT19ISVNUT1JZID0gdHJ1ZVxuZXhwb3J0IGNvbnN0IE5PX1RBQkxFICAgPSBmYWxzZVxuXG4vLyBoYW5kbGVzIHRvIGRhc2hib2FyZCBjb21wb25lbnRzIHRoYXQgY2FuIGJlIHBhc3NlZCBpbnRvIFVzZXIvUG9vbCBvYmplY3RzIC0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tXG4vLyBub3JtYWxseSB3ZSdkIGRvIHRoaXMgd2l0aCBldmVudHMgYnV0IHRoaXMgd2F5IGlzIHNpbXBsZXJcbmV4cG9ydCBpbnRlcmZhY2UgVUlDb250ZXh0IHtcbiAgbG9nOiAgICAgTG9nXG4gIHRhYmxlOiAgIFRhYmxlXG4gIGN1cnJlbnQ6IFBpZUNoYXJ0XG4gIHN0YWNrZWQ6IFN0YWNrZWRQaWVDaGFydFxufVxuXG4vLyBMYWJlbCArIHZhbHVlXG5leHBvcnQgY2xhc3MgRmllbGQge1xuICByb290ICA9IGgoJ2RpdicsIHsgY2xhc3NOYW1lOiAnZmllbGQnIH0pXG4gIGxhYmVsID0gYXBwZW5kKHRoaXMucm9vdCwgaCgnbGFiZWwnKSlcbiAgdmFsdWUgPSBhcHBlbmQodGhpcy5yb290LCBoKCdkaXYnKSlcbiAgY29uc3RydWN0b3IgKG5hbWU6IHN0cmluZywgdmFsdWU/OiBhbnkpIHtcbiAgICB0aGlzLmxhYmVsLnRleHRDb250ZW50ID0gbmFtZVxuICAgIHRoaXMudmFsdWUudGV4dENvbnRlbnQgPSBTdHJpbmcodmFsdWUpXG4gIH1cbiAgYXBwZW5kIChwYXJlbnQ6IEhUTUxFbGVtZW50KSB7XG4gICAgcGFyZW50LmFwcGVuZENoaWxkKHRoaXMucm9vdClcbiAgICByZXR1cm4gdGhpc1xuICB9XG4gIHNldFZhbHVlICh2YWx1ZTogYW55KSB7XG4gICAgdGhpcy52YWx1ZS50ZXh0Q29udGVudCA9IFN0cmluZyh2YWx1ZSlcbiAgfVxufVxuXG4vLyBnbG9iYWwgdmFsdWVzICsgbG9nIG9mIGFsbCBtb2RlbGVkIGV2ZW50cyAtLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tXG5leHBvcnQgY2xhc3MgTG9nIHtcbiAgcm9vdCAgICAgID0gaCgnZGl2JywgeyBjbGFzc05hbWU6ICdoaXN0b3J5JyB9KVxuICBib2R5ICAgICAgPSBhcHBlbmQodGhpcy5yb290LCBoKCdvbCcpKVxuXG4gIG5vdyAgICAgICA9IG5ldyBGaWVsZCgnYmxvY2snKS5hcHBlbmQodGhpcy5yb290KVxuXG4gIGxvY2tlZCAgICA9IG5ldyBGaWVsZCgnbGlxdWlkaXR5IG5vdyBpbiBwb29sJykuYXBwZW5kKHRoaXMucm9vdClcbiAgbGlmZXRpbWUgID0gbmV3IEZpZWxkKCdhbGwgbGlxdWlkaXR5IGV2ZXIgaW4gcG9vbCcpLmFwcGVuZCh0aGlzLnJvb3QpXG5cbiAgYmFsYW5jZSAgID0gbmV3IEZpZWxkKCdhdmFpbGFibGUgcmV3YXJkIGJhbGFuY2UnKS5hcHBlbmQodGhpcy5yb290KVxuICBjbGFpbWVkICAgPSBuZXcgRmllbGQoJ3Jld2FyZHMgY2xhaW1lZCBieSB1c2VycycpLmFwcGVuZCh0aGlzLnJvb3QpXG4gIHJlbWFpbmluZyA9IG5ldyBGaWVsZCgncmVtYWluaW5nIGZ1bmRpbmcgcG9ydGlvbnMnKS5hcHBlbmQodGhpcy5yb290KVxuXG4gIHRocmVzaG9sZCA9IG5ldyBGaWVsZCgnaW5pdGlhbCBhZ2UgdGhyZXNob2xkJykuYXBwZW5kKHRoaXMucm9vdClcbiAgY29vbGRvd24gID0gbmV3IEZpZWxkKCdjb29sZG93biBhZnRlciBjbGFpbScpLmFwcGVuZCh0aGlzLnJvb3QpXG4gIGxpcXVpZCAgICA9IG5ldyBGaWVsZCgncG9vbCBsaXF1aWRpdHkgcmF0aW8nKS5hcHBlbmQodGhpcy5yb290KVxuXG4gIGNsb3NlID0gYXBwZW5kKHRoaXMucm9vdCwgaCgnYnV0dG9uJywgeyB0ZXh0Q29udGVudDogJ2Nsb3NlIHBvb2wnIH0pKVxuXG4gIGFkZCAoZXZlbnQ6IHN0cmluZywgbmFtZTogc3RyaW5nLCBhbW91bnQ6IG51bWJlcnx1bmRlZmluZWQpIHtcbiAgICBpZiAoTk9fSElTVE9SWSkgcmV0dXJuXG4gICAgaWYgKGFtb3VudCkge1xuICAgICAgcHJlcGVuZCh0aGlzLmJvZHksIGgoJ2RpdicsIHsgaW5uZXJIVE1MOiBgPGI+JHtuYW1lfTwvYj4gJHtldmVudH0gJHthbW91bnR9TFBgIH0pKVxuICAgIH0gZWxzZSB7XG4gICAgICBwcmVwZW5kKHRoaXMuYm9keSwgaCgnZGl2JywgeyBpbm5lckhUTUw6IGA8Yj4ke25hbWV9PC9iPiAke2V2ZW50fWAgfSkpXG4gICAgfVxuICB9XG59XG5cbi8vIHRhYmxlIG9mIGN1cnJlbnQgc3RhdGUgLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS1cbmludGVyZmFjZSBDb2x1bW5zIHtcbiAgbmFtZTogICAgICAgICBIVE1MRWxlbWVudFxuICBsYXN0X3VwZGF0ZTogIEhUTUxFbGVtZW50XG4gIGxpZmV0aW1lOiAgICAgSFRNTEVsZW1lbnRcbiAgc2hhcmU6ICAgICAgICBIVE1MRWxlbWVudFxuICBsb2NrZWQ6ICAgICAgIEhUTUxFbGVtZW50XG4gIGxvY2tlZE1pbnVzOiAgSFRNTEVsZW1lbnRcbiAgbG9ja2VkVmFsdWU6ICBIVE1MRWxlbWVudFxuICBsb2NrZWRQbHVzOiAgIEhUTUxFbGVtZW50XG4gIGFnZTogICAgICAgICAgSFRNTEVsZW1lbnRcbiAgZWFybmVkOiAgICAgICBIVE1MRWxlbWVudFxuICBjbGFpbWVkOiAgICAgIEhUTUxFbGVtZW50XG4gIGNsYWltYWJsZTogICAgSFRNTEVsZW1lbnRcbiAgY29vbGRvd246ICAgICBIVE1MRWxlbWVudFxufVxuXG50eXBlIFJvd3MgPSBSZWNvcmQ8c3RyaW5nLCBDb2x1bW5zPlxuXG5leHBvcnQgY2xhc3MgVGFibGUge1xuICByb290OiBIVE1MRWxlbWVudDtcbiAgcm93czogUm93cyA9IHt9O1xuXG4gIGNvbnN0cnVjdG9yICgpIHtcbiAgICB0aGlzLnJvb3QgPSBkb2N1bWVudC5jcmVhdGVFbGVtZW50KCd0YWJsZScpXG4gICAgaWYgKE5PX1RBQkxFKSByZXR1cm5cbiAgfVxuXG4gIGluaXQgKHVzZXJzOiBVc2Vycykge1xuICAgIGFwcGVuZCh0aGlzLnJvb3QsIGgoJ3RoZWFkJywge30sXG4gICAgICBoKCd0aCcsIHsgdGV4dENvbnRlbnQ6ICduYW1lJyAgICAgICAgIH0pLFxuICAgICAgaCgndGgnLCB7IHRleHRDb250ZW50OiAnbGFzdF91cGRhdGUnICB9KSxcbiAgICAgIGgoJ3RoJywgeyB0ZXh0Q29udGVudDogJ2FnZScgICAgICAgICAgfSksXG4gICAgICBoKCd0aCcsIHsgdGV4dENvbnRlbnQ6ICdsb2NrZWQnICAgICAgIH0pLFxuICAgICAgaCgndGgnLCB7IHRleHRDb250ZW50OiAnbGlmZXRpbWUnICAgICB9KSxcbiAgICAgIGgoJ3RoJywgeyB0ZXh0Q29udGVudDogJ3NoYXJlJyAgICAgICAgfSksXG4gICAgICBoKCd0aCcsIHsgdGV4dENvbnRlbnQ6ICdlYXJuZWQnICAgICAgIH0pLFxuICAgICAgaCgndGgnLCB7IHRleHRDb250ZW50OiAnY2xhaW1lZCcgICAgICB9KSxcbiAgICAgIGgoJ3RoJywgeyB0ZXh0Q29udGVudDogJ2NsYWltYWJsZScgICAgfSksXG4gICAgICBoKCd0aCcsIHsgdGV4dENvbnRlbnQ6ICdjb29sZG93bicgICAgIH0pLFxuICAgICkpXG4gICAgZm9yIChjb25zdCBuYW1lIG9mIE9iamVjdC5rZXlzKHVzZXJzKSkge1xuICAgICAgdGhpcy5hZGRSb3cobmFtZSwgdXNlcnNbbmFtZV0pXG4gICAgfVxuICB9XG5cbiAgYWRkUm93IChuYW1lOiBzdHJpbmcsIHVzZXI6IFVzZXIpIHtcbiAgICBpZiAoTk9fVEFCTEUpIHJldHVyblxuICAgIGNvbnN0IHJvdyA9IGFwcGVuZCh0aGlzLnJvb3QsIGgoJ3RyJykpXG4gICAgY29uc3QgbG9ja2VkICAgICAgPSBoKCd0ZCcsIHsgY2xhc3NOYW1lOiAnbG9ja2VkJyB9KVxuICAgICAgICAsIGxvY2tlZE1pbnVzID0gYXBwZW5kKGxvY2tlZCwgaCgnYnV0dG9uJywge1xuICAgICAgICAgICAgICAgICAgICAgICAgICB0ZXh0Q29udGVudDogJy0nLFxuICAgICAgICAgICAgICAgICAgICAgICAgICBvbmNsaWNrOiAoKSA9PiB1c2VyLnJldHJpZXZlKDEwMClcbiAgICAgICAgICAgICAgICAgICAgICAgIH0pKVxuICAgICAgICAsIGxvY2tlZFZhbHVlID0gYXBwZW5kKGxvY2tlZCwgaCgnc3BhbicsIHtcbiAgICAgICAgICAgICAgICAgICAgICAgICAgdGV4dENvbnRlbnQ6ICcnXG4gICAgICAgICAgICAgICAgICAgICAgICB9KSlcbiAgICAgICAgLCBsb2NrZWRQbHVzICA9IGFwcGVuZChsb2NrZWQsIGgoJ2J1dHRvbicsIHtcbiAgICAgICAgICAgICAgICAgICAgICAgICAgdGV4dENvbnRlbnQ6ICcrJyxcbiAgICAgICAgICAgICAgICAgICAgICAgICAgb25jbGljazogKCkgPT4gdXNlci5sb2NrKDEwMClcbiAgICAgICAgICAgICAgICAgICAgICAgIH0pKVxuICAgIGNvbnN0IHJvd3MgPSB0aGlzLnJvd3NbbmFtZV0gPSB7XG4gICAgICBuYW1lOiAgICAgICAgIGFwcGVuZChyb3csIGgoJ3RkJywgeyBzdHlsZTogJ2ZvbnQtd2VpZ2h0OmJvbGQnLCB0ZXh0Q29udGVudDogbmFtZSB9KSksXG4gICAgICBsYXN0X3VwZGF0ZTogIGFwcGVuZChyb3csIGgoJ3RkJykpLFxuICAgICAgYWdlOiAgICAgICAgICBhcHBlbmQocm93LCBoKCd0ZCcpKSxcbiAgICAgIGxvY2tlZDogICAgICAgYXBwZW5kKHJvdywgbG9ja2VkKSxcbiAgICAgIGxvY2tlZE1pbnVzLCBsb2NrZWRWYWx1ZSwgbG9ja2VkUGx1cyxcbiAgICAgIGxpZmV0aW1lOiAgICAgYXBwZW5kKHJvdywgaCgndGQnKSksXG4gICAgICBzaGFyZTogICAgICAgIGFwcGVuZChyb3csIGgoJ3RkJykpLFxuICAgICAgZWFybmVkOiAgICAgICBhcHBlbmQocm93LCBoKCd0ZCcpKSxcbiAgICAgIGNsYWltZWQ6ICAgICAgYXBwZW5kKHJvdywgaCgndGQnKSksXG4gICAgICBjbGFpbWFibGU6ICAgIGFwcGVuZChyb3csIGgoJ3RkJywgeyBjbGFzc05hbWU6ICdjbGFpbWFibGUnLCBvbmNsaWNrOiAoKSA9PiB7dXNlci5jbGFpbSgpfSB9KSksXG4gICAgICBjb29sZG93bjogICAgIGFwcGVuZChyb3csIGgoJ3RkJykpLFxuICAgIH1cbiAgICByb3dzLmNsYWltYWJsZS5zdHlsZS5mb250V2VpZ2h0ID0gJ2JvbGQnXG4gICAgYXBwZW5kKHRoaXMucm9vdCwgcm93KVxuICAgIHJldHVybiByb3dzXG4gIH1cblxuICB1cGRhdGUgKHVzZXI6IFVzZXIpIHtcbiAgICBpZiAoTk9fVEFCTEUpIHJldHVyblxuICAgIHRoaXMucm93c1t1c2VyLm5hbWVdLmxhc3RfdXBkYXRlLnRleHRDb250ZW50ID1cbiAgICAgIGZvcm1hdC5pbnRlZ2VyKHVzZXIubGFzdF91cGRhdGUpXG4gICAgdGhpcy5yb3dzW3VzZXIubmFtZV0ubG9ja2VkVmFsdWUudGV4dENvbnRlbnQgPVxuICAgICAgZm9ybWF0LmludGVnZXIodXNlci5sb2NrZWQpXG4gICAgdGhpcy5yb3dzW3VzZXIubmFtZV0ubGlmZXRpbWUudGV4dENvbnRlbnQgPVxuICAgICAgZm9ybWF0LmludGVnZXIodXNlci5saWZldGltZSlcbiAgICB0aGlzLnJvd3NbdXNlci5uYW1lXS5zaGFyZS50ZXh0Q29udGVudCA9XG4gICAgICBmb3JtYXQucGVyY2VudGFnZSh1c2VyLnNoYXJlKVxuICAgIHRoaXMucm93c1t1c2VyLm5hbWVdLmFnZS50ZXh0Q29udGVudCA9XG4gICAgICBmb3JtYXQuaW50ZWdlcih1c2VyLmFnZSlcbiAgICB0aGlzLnJvd3NbdXNlci5uYW1lXS5lYXJuZWQudGV4dENvbnRlbnQgPVxuICAgICAgZm9ybWF0LmRlY2ltYWwodXNlci5lYXJuZWQpXG4gICAgdGhpcy5yb3dzW3VzZXIubmFtZV0uY2xhaW1lZC50ZXh0Q29udGVudCA9XG4gICAgICBmb3JtYXQuZGVjaW1hbCh1c2VyLmNsYWltZWQpXG4gICAgdGhpcy5yb3dzW3VzZXIubmFtZV0uY2xhaW1hYmxlLnRleHRDb250ZW50ID1cbiAgICAgIGZvcm1hdC5kZWNpbWFsKHVzZXIuY2xhaW1hYmxlKVxuICAgIHRoaXMucm93c1t1c2VyLm5hbWVdLmNvb2xkb3duLnRleHRDb250ZW50ID1cbiAgICAgIGZvcm1hdC5pbnRlZ2VyKHVzZXIuY29vbGRvd24pXG5cbiAgICBjb25zdCBbZmlsbCwgc3Ryb2tlXSA9IHVzZXIuY29sb3JzKClcbiAgICB0aGlzLnJvd3NbdXNlci5uYW1lXS5lYXJuZWQuc3R5bGUuYmFja2dyb3VuZENvbG9yID1cbiAgICB0aGlzLnJvd3NbdXNlci5uYW1lXS5jbGFpbWVkLnN0eWxlLmJhY2tncm91bmRDb2xvciA9XG4gICAgdGhpcy5yb3dzW3VzZXIubmFtZV0uY2xhaW1hYmxlLnN0eWxlLmJhY2tncm91bmRDb2xvciA9XG4gICAgICBmaWxsXG4gICAgdGhpcy5yb3dzW3VzZXIubmFtZV0uY2xhaW1hYmxlLnN0eWxlLmNvbG9yID1cbiAgICAgIHN0cm9rZVxuICB9XG59XG5cbnR5cGUgVmFsdWVzID0gUmVjb3JkPHN0cmluZywgbnVtYmVyPlxuZXhwb3J0IGNsYXNzIFBpZUNoYXJ0IHtcbiAgcm9vdDogICBIVE1MRWxlbWVudDtcbiAgY2FudmFzOiBIVE1MQ2FudmFzRWxlbWVudDtcblxuICB1c2VyczogVXNlcnMgPSB7fTtcbiAgdG90YWw6IG51bWJlciA9IDA7XG4gIGZpZWxkOiBzdHJpbmc7XG5cbiAgY29uc3RydWN0b3IgKF9uYW1lOiBzdHJpbmcsIGZpZWxkOiBzdHJpbmcpIHtcbiAgICB0aGlzLmZpZWxkICA9IGZpZWxkXG4gICAgdGhpcy5yb290ICAgPSBoKCdkaXYnLCB7IGNsYXNzTmFtZTogYHBpZSAke2ZpZWxkfWAgfSlcbiAgICB0aGlzLmNhbnZhcyA9IGFwcGVuZCh0aGlzLnJvb3QsIGgoJ2NhbnZhcycsIHsgd2lkdGg6IDEsIGhlaWdodDogMSB9KSkgYXMgSFRNTENhbnZhc0VsZW1lbnRcbiAgfVxuXG4gIGFkZCAodXNlcjogVXNlcikge1xuICAgIHRoaXMudXNlcnNbdXNlci5uYW1lXSA9IHVzZXJcbiAgfVxuXG4gIHJlbW92ZSAodXNlcjogVXNlcikge1xuICAgIGRlbGV0ZSB0aGlzLnVzZXJzW3VzZXIubmFtZV1cbiAgfVxuXG4gIHJlc2l6ZSAoKSB7XG4gICAgdGhpcy5jYW52YXMud2lkdGggPSB0aGlzLmNhbnZhcy5oZWlnaHQgPSAxXG4gICAgY29uc3Qgc2l6ZSA9IE1hdGgubWluKHRoaXMucm9vdC5vZmZzZXRXaWR0aCwgdGhpcy5yb290Lm9mZnNldEhlaWdodClcbiAgICB0aGlzLmNhbnZhcy53aWR0aCA9IHRoaXMuY2FudmFzLmhlaWdodCA9IHNpemVcbiAgICB0aGlzLnJlbmRlcigpXG4gIH1cblxuICByZW5kZXIgKCkge1xuICAgIHJlcXVlc3RBbmltYXRpb25GcmFtZSgoKT0+e1xuICAgICAgLy8gZXh0cmFjdCBuZWVkZWQgZGF0dW0gZnJvbSB1c2VyIGxpc3RcbiAgICAgIC8vIGFuZCBzdW0gdGhlIHRvdGFsXG4gICAgICBjb25zdCB2YWx1ZXM6IFZhbHVlcyA9IHt9XG4gICAgICBsZXQgdG90YWw6IG51bWJlciA9IDBcbiAgICAgIGZvciAoY29uc3QgdXNlciBvZiBPYmplY3QudmFsdWVzKHRoaXMudXNlcnMpKSB7XG4gICAgICAgIGNvbnN0IHZhbHVlID0gKHVzZXIgYXMgYW55KVt0aGlzLmZpZWxkXVxuICAgICAgICBpZiAodmFsdWUpIHtcbiAgICAgICAgICB0b3RhbCArPSB2YWx1ZVxuICAgICAgICAgIHZhbHVlc1t1c2VyLm5hbWVdID0gdmFsdWUgfSB9XG4gICAgICBpZiAodG90YWwgPT09IDApIHJldHVyblxuXG4gICAgICAvLyBwcmVwYXJlIGNhbnZhc1xuICAgICAgY29uc3Qge3dpZHRoLCBoZWlnaHR9ID0gdGhpcy5jYW52YXNcbiAgICAgIGNvbnN0IGNvbnRleHQgPSB0aGlzLmNhbnZhcy5nZXRDb250ZXh0KCcyZCcpIGFzIENhbnZhc1JlbmRlcmluZ0NvbnRleHQyRDtcblxuICAgICAgLy8gY2xlYXJcbiAgICAgIGNvbnRleHQuZmlsbFN0eWxlID0gJyMyODI4MjgnXG4gICAgICBjb250ZXh0LmZpbGxSZWN0KDEsIDEsIHdpZHRoLTIsIGhlaWdodC0yKVxuXG4gICAgICAvLyBkZWZpbmUgY2VudGVyXG4gICAgICBjb25zdCBjZW50ZXJYID0gd2lkdGggIC8gMlxuICAgICAgY29uc3QgY2VudGVyWSA9IGhlaWdodCAvIDJcbiAgICAgIGNvbnN0IHJhZGl1cyAgPSBjZW50ZXJYICogMC45NVxuXG4gICAgICAvLyBsb29wIG92ZXIgc2VnbWVudHNcbiAgICAgIGxldCBzdGFydCA9IDBcbiAgICAgIGZvciAoY29uc3QgbmFtZSBvZiBPYmplY3Qua2V5cyh0aGlzLnVzZXJzKS5zb3J0KCkpIHtcbiAgICAgICAgY29uc3QgdmFsdWUgPSB2YWx1ZXNbbmFtZV1cbiAgICAgICAgaWYgKHZhbHVlKSB7XG4gICAgICAgICAgY29uc3QgcG9ydGlvbiA9IHZhbHVlIC8gdG90YWxcbiAgICAgICAgICBjb25zdCBlbmQgICAgID0gc3RhcnQgKyAoMipwb3J0aW9uKVxuICAgICAgICAgIGNvbnRleHQuYmVnaW5QYXRoKClcbiAgICAgICAgICBjb250ZXh0Lm1vdmVUbyhjZW50ZXJYLCBjZW50ZXJZKVxuICAgICAgICAgIGNvbnRleHQuYXJjKGNlbnRlclgsIGNlbnRlclksIHJhZGl1cywgc3RhcnQgKiBNYXRoLlBJLCBlbmQgKiBNYXRoLlBJKVxuICAgICAgICAgIC8vY29udGV4dC5tb3ZlVG8oY2VudGVyWCwgY2VudGVyWSlcbiAgICAgICAgICBjb25zdCBbZmlsbFN0eWxlLCBzdHJva2VTdHlsZV0gPSB0aGlzLnVzZXJzW25hbWVdLmNvbG9ycygpXG4gICAgICAgICAgY29udGV4dC5maWxsU3R5bGUgPSBmaWxsU3R5bGVcbiAgICAgICAgICBjb250ZXh0LmxpbmVXaWR0aCA9IDAuOFxuICAgICAgICAgIGNvbnRleHQuc3Ryb2tlU3R5bGUgPSBzdHJva2VTdHlsZS8vICcjMDAwJy8vcmdiYSgyNTUsMjU1LDI1NSwwLjUpJ1xuICAgICAgICAgIGNvbnRleHQuZmlsbCgpXG4gICAgICAgICAgY29udGV4dC5zdHJva2UoKVxuICAgICAgICAgIHN0YXJ0ID0gZW5kIH0gfSB9KSB9IH1cblxuZXhwb3J0IGNsYXNzIFN0YWNrZWRQaWVDaGFydCB7XG4gIHJvb3Q6ICAgSFRNTEVsZW1lbnQ7XG4gIGNhbnZhczogSFRNTENhbnZhc0VsZW1lbnQ7XG5cbiAgdXNlcnM6IFVzZXJzID0ge307XG4gIGFkZCAodXNlcjogVXNlcikge1xuICAgIHRoaXMudXNlcnNbdXNlci5uYW1lXSA9IHVzZXIgfVxuICByZW1vdmUgKHVzZXI6IFVzZXIpIHtcbiAgICBkZWxldGUgdGhpcy51c2Vyc1t1c2VyLm5hbWVdIH1cblxuICBjb25zdHJ1Y3RvciAoKSB7XG4gICAgdGhpcy5yb290ICAgPSBoKCdkaXYnLCB7IGNsYXNzTmFtZTogYHBpZSBzdGFja2VkYCB9KVxuICAgIHRoaXMuY2FudmFzID0gYXBwZW5kKHRoaXMucm9vdCwgaCgnY2FudmFzJywgeyB3aWR0aDogMSwgaGVpZ2h0OiAxIH0pKSBhcyBIVE1MQ2FudmFzRWxlbWVudCB9XG5cbiAgcmVzaXplICgpIHtcbiAgICB0aGlzLmNhbnZhcy53aWR0aCA9IHRoaXMuY2FudmFzLmhlaWdodCA9IDFcbiAgICBjb25zdCBzaXplID0gTWF0aC5taW4odGhpcy5yb290Lm9mZnNldFdpZHRoLCB0aGlzLnJvb3Qub2Zmc2V0SGVpZ2h0KVxuICAgIHRoaXMuY2FudmFzLndpZHRoID0gdGhpcy5jYW52YXMuaGVpZ2h0ID0gc2l6ZVxuICAgIHRoaXMucmVuZGVyKCkgfVxuXG4gIHJlbmRlciAoKSB7XG4gICAgcmVxdWVzdEFuaW1hdGlvbkZyYW1lKCgpPT57XG4gICAgICAvLyBleHRyYWN0IG5lZWRlZCBkYXR1bSBmcm9tIHVzZXIgbGlzdFxuICAgICAgLy8gYW5kIHN1bSB0aGUgdG90YWxcbiAgICAgIGxldCB0b3RhbDogbnVtYmVyID0gMFxuICAgICAgZm9yIChjb25zdCB1c2VyIG9mIE9iamVjdC52YWx1ZXModGhpcy51c2VycykpIHtcbiAgICAgICAgdG90YWwgKz0gdXNlci5saWZldGltZVxuICAgICAgfVxuICAgICAgaWYgKHRvdGFsID09PSAwKSByZXR1cm5cblxuICAgICAgLy8gcHJlcGFyZSBjYW52YXNcbiAgICAgIGNvbnN0IHt3aWR0aCwgaGVpZ2h0fSA9IHRoaXMuY2FudmFzXG4gICAgICBjb25zdCBjb250ZXh0ID0gdGhpcy5jYW52YXMuZ2V0Q29udGV4dCgnMmQnKSBhcyBDYW52YXNSZW5kZXJpbmdDb250ZXh0MkQ7XG5cbiAgICAgIC8vIGNsZWFyXG4gICAgICBjb250ZXh0LmZpbGxTdHlsZSA9ICcjMjgyODI4J1xuICAgICAgY29udGV4dC5maWxsUmVjdCgxLCAxLCB3aWR0aC0yLCBoZWlnaHQtMilcblxuICAgICAgLy8gZGVmaW5lIGNlbnRlclxuICAgICAgY29uc3QgY2VudGVyWCA9IHdpZHRoICAvIDJcbiAgICAgIGNvbnN0IGNlbnRlclkgPSBoZWlnaHQgLyAyXG4gICAgICBjb25zdCByYWRpdXMgID0gY2VudGVyWCAqIDAuOTVcblxuICAgICAgLy8gbG9vcCBvdmVyIHNlZ21lbnRzXG4gICAgICBsZXQgc3RhcnQgPSAwXG4gICAgICBmb3IgKGNvbnN0IG5hbWUgb2YgT2JqZWN0LmtleXModGhpcy51c2Vycykuc29ydCgpKSB7XG4gICAgICAgIGNvbnN0IHVzZXIgPSB0aGlzLnVzZXJzW25hbWVdXG4gICAgICAgIGlmICh1c2VyLmxpZmV0aW1lID09PSAwKSBjb250aW51ZVxuICAgICAgICBjb25zdCBwb3J0aW9uID0gdXNlci5saWZldGltZSAvIHRvdGFsXG4gICAgICAgIGNvbnN0IGVuZCAgICAgPSBzdGFydCArICgyKnBvcnRpb24pXG4gICAgICAgIGNvbnRleHQuYmVnaW5QYXRoKClcbiAgICAgICAgY29udGV4dC5tb3ZlVG8oY2VudGVyWCwgY2VudGVyWSlcbiAgICAgICAgY29udGV4dC5hcmMoY2VudGVyWCwgY2VudGVyWSwgcmFkaXVzLCBzdGFydCAqIE1hdGguUEksIGVuZCAqIE1hdGguUEkpXG4gICAgICAgIC8vY29udGV4dC5tb3ZlVG8oY2VudGVyWCwgY2VudGVyWSlcbiAgICAgICAgY29uc3QgW2ZpbGxTdHlsZSwgc3Ryb2tlU3R5bGVdID0gdXNlci5jb2xvcnMoKVxuICAgICAgICBjb250ZXh0LmZpbGxTdHlsZSA9IGZpbGxTdHlsZVxuICAgICAgICBjb250ZXh0LnN0cm9rZVN0eWxlID0gc3Ryb2tlU3R5bGUvLycjMDAwJy8vJ3JnYmEoMjU1LDI1NSwyNTUsMC41KSdcbiAgICAgICAgLy9jb250ZXh0LnN0cm9rZVN0eWxlID0gZmlsbFN0eWxlLy9zdHJva2VTdHlsZVxuICAgICAgICBjb250ZXh0LmxpbmVXaWR0aCA9IDAuOFxuICAgICAgICBjb250ZXh0LmZpbGwoKVxuICAgICAgICBjb250ZXh0LnN0cm9rZSgpXG4gICAgICAgIHN0YXJ0ID0gZW5kIH0gfSkgfVxuXG59XG4iLCIvLyBUaGUgbW9kdWxlIGNhY2hlXG52YXIgX193ZWJwYWNrX21vZHVsZV9jYWNoZV9fID0ge307XG5cbi8vIFRoZSByZXF1aXJlIGZ1bmN0aW9uXG5mdW5jdGlvbiBfX3dlYnBhY2tfcmVxdWlyZV9fKG1vZHVsZUlkKSB7XG5cdC8vIENoZWNrIGlmIG1vZHVsZSBpcyBpbiBjYWNoZVxuXHR2YXIgY2FjaGVkTW9kdWxlID0gX193ZWJwYWNrX21vZHVsZV9jYWNoZV9fW21vZHVsZUlkXTtcblx0aWYgKGNhY2hlZE1vZHVsZSAhPT0gdW5kZWZpbmVkKSB7XG5cdFx0cmV0dXJuIGNhY2hlZE1vZHVsZS5leHBvcnRzO1xuXHR9XG5cdC8vIENyZWF0ZSBhIG5ldyBtb2R1bGUgKGFuZCBwdXQgaXQgaW50byB0aGUgY2FjaGUpXG5cdHZhciBtb2R1bGUgPSBfX3dlYnBhY2tfbW9kdWxlX2NhY2hlX19bbW9kdWxlSWRdID0ge1xuXHRcdGlkOiBtb2R1bGVJZCxcblx0XHQvLyBubyBtb2R1bGUubG9hZGVkIG5lZWRlZFxuXHRcdGV4cG9ydHM6IHt9XG5cdH07XG5cblx0Ly8gRXhlY3V0ZSB0aGUgbW9kdWxlIGZ1bmN0aW9uXG5cdF9fd2VicGFja19tb2R1bGVzX19bbW9kdWxlSWRdKG1vZHVsZSwgbW9kdWxlLmV4cG9ydHMsIF9fd2VicGFja19yZXF1aXJlX18pO1xuXG5cdC8vIFJldHVybiB0aGUgZXhwb3J0cyBvZiB0aGUgbW9kdWxlXG5cdHJldHVybiBtb2R1bGUuZXhwb3J0cztcbn1cblxuLy8gZXhwb3NlIHRoZSBtb2R1bGVzIG9iamVjdCAoX193ZWJwYWNrX21vZHVsZXNfXylcbl9fd2VicGFja19yZXF1aXJlX18ubSA9IF9fd2VicGFja19tb2R1bGVzX187XG5cbiIsIi8vIGdldERlZmF1bHRFeHBvcnQgZnVuY3Rpb24gZm9yIGNvbXBhdGliaWxpdHkgd2l0aCBub24taGFybW9ueSBtb2R1bGVzXG5fX3dlYnBhY2tfcmVxdWlyZV9fLm4gPSAobW9kdWxlKSA9PiB7XG5cdHZhciBnZXR0ZXIgPSBtb2R1bGUgJiYgbW9kdWxlLl9fZXNNb2R1bGUgP1xuXHRcdCgpID0+IChtb2R1bGVbJ2RlZmF1bHQnXSkgOlxuXHRcdCgpID0+IChtb2R1bGUpO1xuXHRfX3dlYnBhY2tfcmVxdWlyZV9fLmQoZ2V0dGVyLCB7IGE6IGdldHRlciB9KTtcblx0cmV0dXJuIGdldHRlcjtcbn07IiwiLy8gZGVmaW5lIGdldHRlciBmdW5jdGlvbnMgZm9yIGhhcm1vbnkgZXhwb3J0c1xuX193ZWJwYWNrX3JlcXVpcmVfXy5kID0gKGV4cG9ydHMsIGRlZmluaXRpb24pID0+IHtcblx0Zm9yKHZhciBrZXkgaW4gZGVmaW5pdGlvbikge1xuXHRcdGlmKF9fd2VicGFja19yZXF1aXJlX18ubyhkZWZpbml0aW9uLCBrZXkpICYmICFfX3dlYnBhY2tfcmVxdWlyZV9fLm8oZXhwb3J0cywga2V5KSkge1xuXHRcdFx0T2JqZWN0LmRlZmluZVByb3BlcnR5KGV4cG9ydHMsIGtleSwgeyBlbnVtZXJhYmxlOiB0cnVlLCBnZXQ6IGRlZmluaXRpb25ba2V5XSB9KTtcblx0XHR9XG5cdH1cbn07IiwiX193ZWJwYWNrX3JlcXVpcmVfXy5nID0gKGZ1bmN0aW9uKCkge1xuXHRpZiAodHlwZW9mIGdsb2JhbFRoaXMgPT09ICdvYmplY3QnKSByZXR1cm4gZ2xvYmFsVGhpcztcblx0dHJ5IHtcblx0XHRyZXR1cm4gdGhpcyB8fCBuZXcgRnVuY3Rpb24oJ3JldHVybiB0aGlzJykoKTtcblx0fSBjYXRjaCAoZSkge1xuXHRcdGlmICh0eXBlb2Ygd2luZG93ID09PSAnb2JqZWN0JykgcmV0dXJuIHdpbmRvdztcblx0fVxufSkoKTsiLCJfX3dlYnBhY2tfcmVxdWlyZV9fLm8gPSAob2JqLCBwcm9wKSA9PiAoT2JqZWN0LnByb3RvdHlwZS5oYXNPd25Qcm9wZXJ0eS5jYWxsKG9iaiwgcHJvcCkpIiwiLy8gZGVmaW5lIF9fZXNNb2R1bGUgb24gZXhwb3J0c1xuX193ZWJwYWNrX3JlcXVpcmVfXy5yID0gKGV4cG9ydHMpID0+IHtcblx0aWYodHlwZW9mIFN5bWJvbCAhPT0gJ3VuZGVmaW5lZCcgJiYgU3ltYm9sLnRvU3RyaW5nVGFnKSB7XG5cdFx0T2JqZWN0LmRlZmluZVByb3BlcnR5KGV4cG9ydHMsIFN5bWJvbC50b1N0cmluZ1RhZywgeyB2YWx1ZTogJ01vZHVsZScgfSk7XG5cdH1cblx0T2JqZWN0LmRlZmluZVByb3BlcnR5KGV4cG9ydHMsICdfX2VzTW9kdWxlJywgeyB2YWx1ZTogdHJ1ZSB9KTtcbn07IiwidmFyIHNjcmlwdFVybDtcbmlmIChfX3dlYnBhY2tfcmVxdWlyZV9fLmcuaW1wb3J0U2NyaXB0cykgc2NyaXB0VXJsID0gX193ZWJwYWNrX3JlcXVpcmVfXy5nLmxvY2F0aW9uICsgXCJcIjtcbnZhciBkb2N1bWVudCA9IF9fd2VicGFja19yZXF1aXJlX18uZy5kb2N1bWVudDtcbmlmICghc2NyaXB0VXJsICYmIGRvY3VtZW50KSB7XG5cdGlmIChkb2N1bWVudC5jdXJyZW50U2NyaXB0KVxuXHRcdHNjcmlwdFVybCA9IGRvY3VtZW50LmN1cnJlbnRTY3JpcHQuc3JjXG5cdGlmICghc2NyaXB0VXJsKSB7XG5cdFx0dmFyIHNjcmlwdHMgPSBkb2N1bWVudC5nZXRFbGVtZW50c0J5VGFnTmFtZShcInNjcmlwdFwiKTtcblx0XHRpZihzY3JpcHRzLmxlbmd0aCkgc2NyaXB0VXJsID0gc2NyaXB0c1tzY3JpcHRzLmxlbmd0aCAtIDFdLnNyY1xuXHR9XG59XG4vLyBXaGVuIHN1cHBvcnRpbmcgYnJvd3NlcnMgd2hlcmUgYW4gYXV0b21hdGljIHB1YmxpY1BhdGggaXMgbm90IHN1cHBvcnRlZCB5b3UgbXVzdCBzcGVjaWZ5IGFuIG91dHB1dC5wdWJsaWNQYXRoIG1hbnVhbGx5IHZpYSBjb25maWd1cmF0aW9uXG4vLyBvciBwYXNzIGFuIGVtcHR5IHN0cmluZyAoXCJcIikgYW5kIHNldCB0aGUgX193ZWJwYWNrX3B1YmxpY19wYXRoX18gdmFyaWFibGUgZnJvbSB5b3VyIGNvZGUgdG8gdXNlIHlvdXIgb3duIGxvZ2ljLlxuaWYgKCFzY3JpcHRVcmwpIHRocm93IG5ldyBFcnJvcihcIkF1dG9tYXRpYyBwdWJsaWNQYXRoIGlzIG5vdCBzdXBwb3J0ZWQgaW4gdGhpcyBicm93c2VyXCIpO1xuc2NyaXB0VXJsID0gc2NyaXB0VXJsLnJlcGxhY2UoLyMuKiQvLCBcIlwiKS5yZXBsYWNlKC9cXD8uKiQvLCBcIlwiKS5yZXBsYWNlKC9cXC9bXlxcL10rJC8sIFwiL1wiKTtcbl9fd2VicGFja19yZXF1aXJlX18ucCA9IHNjcmlwdFVybDsiLCJfX3dlYnBhY2tfcmVxdWlyZV9fLmIgPSBkb2N1bWVudC5iYXNlVVJJIHx8IHNlbGYubG9jYXRpb24uaHJlZjtcblxuLy8gb2JqZWN0IHRvIHN0b3JlIGxvYWRlZCBhbmQgbG9hZGluZyBjaHVua3Ncbi8vIHVuZGVmaW5lZCA9IGNodW5rIG5vdCBsb2FkZWQsIG51bGwgPSBjaHVuayBwcmVsb2FkZWQvcHJlZmV0Y2hlZFxuLy8gW3Jlc29sdmUsIHJlamVjdCwgUHJvbWlzZV0gPSBjaHVuayBsb2FkaW5nLCAwID0gY2h1bmsgbG9hZGVkXG52YXIgaW5zdGFsbGVkQ2h1bmtzID0ge1xuXHRcIm1haW5cIjogMFxufTtcblxuLy8gbm8gY2h1bmsgb24gZGVtYW5kIGxvYWRpbmdcblxuLy8gbm8gcHJlZmV0Y2hpbmdcblxuLy8gbm8gcHJlbG9hZGVkXG5cbi8vIG5vIEhNUlxuXG4vLyBubyBITVIgbWFuaWZlc3RcblxuLy8gbm8gb24gY2h1bmtzIGxvYWRlZFxuXG4vLyBubyBqc29ucCBmdW5jdGlvbiIsImltcG9ydCAnLi9kYXNoYm9hcmQvc3R5bGUuY3NzJ1xuaW1wb3J0IHsgTG9nLCBUYWJsZSwgUGllQ2hhcnQsIFN0YWNrZWRQaWVDaGFydCB9IGZyb20gJy4vZGFzaGJvYXJkL3dpZGdldHMnXG5pbXBvcnQgeyBULCBVc2VycywgTUFYX1VTRVJTLCBNQVhfSU5JVElBTCB9IGZyb20gJy4vZGFzaGJvYXJkL2NvbnRyYWN0X2Jhc2UnXG5pbXBvcnQgeyBSZWFsUG9vbCBhcyBQb29sLCBSZWFsVXNlciBhcyBVc2VyIH0gZnJvbSAnLi9kYXNoYm9hcmQvY29udHJhY3RfcmVhbCdcbmltcG9ydCB7IHJhbmRvbSwgcGlja1JhbmRvbSwgdGhyb3R0bGUsIGFmdGVyLCBhcHBlbmQgfSBmcm9tICcuL2Rhc2hib2FyZC9oZWxwZXJzJ1xuLy9pbXBvcnQgaW5pdE1vY2sgZnJvbSAnLi9kYXNoYm9hcmQvY29udHJhY3RfbW9jaydcbmltcG9ydCBpbml0UmVhbCBmcm9tICcuL2Rhc2hib2FyZC9jb250cmFjdF9yZWFsJ1xuXG5kb2N1bWVudC5ib2R5LmlubmVySFRNTCA9ICc8Y2VudGVyPmxvYWRpbmc8L2NlbnRlcj4nXG5cbi8vIHNldHRpbmdzIC0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS1cbmNvbnN0IFVQREFURV9JTlRFUlZBTCAgPSAxXG5jb25zdCBBVVRPX0NMQUlNICAgICAgID0gZmFsc2VcbmNvbnN0IEFVVE9fTE9DS19VTkxPQ0sgPSBmYWxzZVxuXG5pbml0UmVhbCgpLnRoZW4oKCk9PnsgLy8gbG9hZCB0aGVuIHN0YXJ0IG9uIGNsaWNrIC0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tXG4gIGRvY3VtZW50LmJvZHkub25jbGljayA9ICgpID0+IHtcbiAgICBkb2N1bWVudC5ib2R5LmlubmVySFRNTCA9ICcnXG4gICAgZG9jdW1lbnQuYm9keS5vbmNsaWNrID0gbnVsbFxuICAgIHN0YXJ0KClcbiAgfVxuICBkb2N1bWVudC5ib2R5LmlubmVySFRNTCA9ICc8Y2VudGVyPmNsaWNrIHRvIHN0YXJ0PC9jZW50ZXI+J1xufSlcblxuZnVuY3Rpb24gc3RhcnQgKCkge1xuXG4gIC8vIGNyZWF0ZSB0aGUgZGFzaGJvYXJkIC0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tXG4gIGNvbnN0IHVpID0ge1xuICAgIGxvZzogICAgIG5ldyBMb2coKSxcbiAgICB0YWJsZTogICBuZXcgVGFibGUoKSxcbiAgICBjdXJyZW50OiBuZXcgUGllQ2hhcnQoJ0N1cnJlbnQgYW1vdW50cyBsb2NrZWQnLCAgJ2xvY2tlZCcpLFxuICAgIHN0YWNrZWQ6IG5ldyBTdGFja2VkUGllQ2hhcnQoKVxuICB9XG5cbiAgLy8gY3JlYXRlIGEgcG9vbCBhbmQgc29tZSBvZiB0ZXN0IHVzZXJzIHdpdGggcmFuZG9tIGJhbGFuY2VzIC0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS1cbiAgY29uc3QgcG9vbCA9IG5ldyBQb29sKHVpKVxuICBjb25zdCB1c2VyczogVXNlcnMgPSB7fVxuICBmb3IgKGxldCBpID0gMDsgaSA8IE1BWF9VU0VSUzsgaSsrKSB7XG4gICAgY29uc3QgbmFtZSAgICA9IGBVc2VyJHtpfWBcbiAgICBjb25zdCBiYWxhbmNlID0gTWF0aC5mbG9vcihNYXRoLnJhbmRvbSgpKk1BWF9JTklUSUFMKVxuICAgIHVzZXJzW25hbWVdICAgPSBuZXcgVXNlcih1aSwgcG9vbCwgbmFtZSwgYmFsYW5jZSlcbiAgfVxuXG4gIC8vIGFkZCBjb21wb25lbnRzIC0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tXG4gIGZvciAoY29uc3QgZWwgb2YgT2JqZWN0LnZhbHVlcyh1aSkpIHtcbiAgICBhcHBlbmQoZG9jdW1lbnQuYm9keSwgZWwucm9vdClcbiAgfVxuXG4gIC8vIGNyZWF0ZSBkb20gZWxlbWVudHMgZm9yIGFsbCB1c2VycyAtIHRoZW4gb25seSB1cGRhdGUgdGhlIGNvbnRlbnQgLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tXG4gIHVpLnRhYmxlLmluaXQodXNlcnMpXG5cbiAgLy8gYWRkIHJlc2l6ZSBoYW5kbGVyIC0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS1cbiAgcmVzaXplKClcbiAgd2luZG93LmFkZEV2ZW50TGlzdGVuZXIoJ3Jlc2l6ZScsIHRocm90dGxlKDEwMCwgcmVzaXplKSlcblxuICAvLyBzdGFydCB1cGRhdGluZyAtLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLVxuICB1cGRhdGUoKVxuICBmdW5jdGlvbiB1cGRhdGUgKCkge1xuICAgIC8vIGFkdmFuY2UgdGltZSAtLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLVxuICAgIFQuVCsrXG4gICAgcG9vbC5jb250cmFjdC5ibG9jayA9IFQuVFxuXG4gICAgLy8gcGVyaW9kaWNhbGx5IGZ1bmQgcG9vbCBhbmQgaW5jcmVtZW50IGl0cyBsaWZldGltZSAtLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tXG4gICAgcG9vbC51cGRhdGUoKVxuXG4gICAgLy8gaW5jcmVtZW50IGxpZmV0aW1lcyBhbmQgYWdlczsgY29sbGVjdCBlbGlnaWJsZSBjbGFpbWFudHMgLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tXG4gICAgY29uc3QgZWxpZ2libGU6IEFycmF5PFVzZXI+ID0gW11cbiAgICBmb3IgKGNvbnN0IHVzZXIgb2YgT2JqZWN0LnZhbHVlcyh1c2VycykpIHtcbiAgICAgIHVzZXIudXBkYXRlKClcbiAgICAgIGlmICh1c2VyLmNsYWltYWJsZSA+IDApIGVsaWdpYmxlLnB1c2godXNlciBhcyBVc2VyKVxuICAgIH1cblxuICAgIC8vIHBlcmZvcm0gcmFuZG9tIGxvY2svcmV0cmlldmUgZnJvbSByYW5kb20gYWNjb3VudCBmb3IgcmFuZG9tIGFtb3VudCAtLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLVxuICAgIGlmIChBVVRPX0xPQ0tfVU5MT0NLKSB7XG4gICAgICBjb25zdCB1c2VyID0gcGlja1JhbmRvbShPYmplY3QudmFsdWVzKHVzZXJzKSlcbiAgICAgIHBpY2tSYW5kb20oW1xuICAgICAgICAoYW1vdW50Om51bWJlcik9PnVzZXIubG9jayhhbW91bnQpLFxuICAgICAgICAoYW1vdW50Om51bWJlcik9PnVzZXIucmV0cmlldmUoYW1vdW50KVxuICAgICAgXSkocmFuZG9tKHVzZXIuYmFsYW5jZSkpXG4gICAgfVxuXG4gICAgLy8gcGVyZm9ybSByYW5kb20gY2xhaW0gLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tXG4gICAgaWYgKEFVVE9fQ0xBSU0gJiYgZWxpZ2libGUubGVuZ3RoID4gMCkge1xuICAgICAgY29uc3QgY2xhaW1hbnQgPSBwaWNrUmFuZG9tKGVsaWdpYmxlKVxuICAgICAgY2xhaW1hbnQuY2xhaW0oKVxuICAgIH1cblxuICAgIC8vIHVwZGF0ZSBjaGFydHMgLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLVxuICAgIGZvciAoY29uc3QgY2hhcnQgb2YgW3VpLmN1cnJlbnQsdWkuc3RhY2tlZF0pIHtcbiAgICAgIGNoYXJ0LnJlbmRlcigpXG4gICAgfVxuXG4gICAgLy8gcmluc2UgYW5kIHJlcGVhdCAtLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tXG4gICAgYWZ0ZXIoVVBEQVRFX0lOVEVSVkFMLCB1cGRhdGUpXG4gIH1cblxuICAvLyByZXNpemUgaGFuZGxlciAtLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLVxuICBmdW5jdGlvbiByZXNpemUgKCkge1xuICAgIHVpLmN1cnJlbnQucmVzaXplKClcbiAgICB1aS5zdGFja2VkLnJlc2l6ZSgpXG4gIH1cbn1cbiJdLCJzb3VyY2VSb290IjoiIn0=