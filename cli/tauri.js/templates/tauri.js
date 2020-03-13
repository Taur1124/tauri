/* eslint-disable */

/**
 *  * THIS FILE IS GENERATED AUTOMATICALLY.
 * DO NOT EDIT.
 *
 * Please whitelist these API functions in tauri.conf.json
 *
 **/

/**
 * @module tauri
 * @description This API interface makes powerful interactions available
 * to be run on client side applications. They are opt-in features, and
 * must be enabled in tauri.conf.json
 *
 * Each binding MUST provide these interfaces in order to be compliant,
 * and also whitelist them based upon the developer's settings.
 */

// polyfills
if (!String.prototype.startsWith) {
  String.prototype.startsWith = function (searchString, position) {
    position = position || 0
    return this.substr(position, searchString.length) === searchString
  }
}

// makes the window.external.invoke API available after window.location.href changes

switch (navigator.platform) {
  case "Macintosh":
  case "MacPPC":
  case "MacIntel":
  case "Mac68K":
    window.external = this
    invoke = function (x) {
      webkit.messageHandlers.invoke.postMessage(x);
    }
    break;
  case "Windows":
  case "WinCE":
  case "Win32":
  case "Win64":
    break;
  default: 
    window.external = this
    invoke = function (x) {
      window.webkit.messageHandlers.external.postMessage(x);
    }
    break;
}


function s4() {
  return Math.floor((1 + Math.random()) * 0x10000)
    .toString(16)
    .substring(1)
}

var uid = function () {
  return s4() + s4() + '-' + s4() + '-' + s4() + '-' +
    s4() + '-' + s4() + s4() + s4()
}

function ownKeys(object, enumerableOnly) { var keys = Object.keys(object); if (Object.getOwnPropertySymbols) { var symbols = Object.getOwnPropertySymbols(object); if (enumerableOnly) symbols = symbols.filter(function (sym) { return Object.getOwnPropertyDescriptor(object, sym).enumerable; }); keys.push.apply(keys, symbols); } return keys; }

function _objectSpread(target) { for (var i = 1; i < arguments.length; i++) { var source = arguments[i] != null ? arguments[i] : {}; if (i % 2) { ownKeys(source, true).forEach(function (key) { _defineProperty(target, key, source[key]); }); } else if (Object.getOwnPropertyDescriptors) { Object.defineProperties(target, Object.getOwnPropertyDescriptors(source)); } else { ownKeys(source).forEach(function (key) { Object.defineProperty(target, key, Object.getOwnPropertyDescriptor(source, key)); }); } } return target; }

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }


function _typeof(obj) { if (typeof Symbol === "function" && typeof Symbol.iterator === "symbol") { _typeof = function _typeof(obj) { return typeof obj; }; } else { _typeof = function _typeof(obj) { return obj && typeof Symbol === "function" && obj.constructor === Symbol && obj !== Symbol.prototype ? "symbol" : typeof obj; }; } return _typeof(obj); }

<% if (ctx.dev) { %>
/**
 * @name return __whitelistWarning
 * @description Present a stylish warning to the developer that their API
 * call has not been whitelisted in tauri.conf.json
 * @param {String} func - function name to warn
 * @private
 */
var __whitelistWarning = function (func) {
    console.warn('%c[Tauri] Danger \ntauri.' + func + ' not whitelisted 💣\n%c\nAdd to tauri.conf.json: \n\ntauri: \n  whitelist: { \n    ' + func + ': true \n\nReference: https://github.com/tauri-apps/tauri/wiki' + func, 'background: red; color: white; font-weight: 800; padding: 2px; font-size:1.5em', ' ')
    return __reject()
  }
    <% } %>

<% if (ctx.dev) { %>
  /**
   * @name __reject
   * @description generates a promise used to deflect un-whitelisted tauri API calls
   * Its only purpose is to maintain thenable structure in client code without
   * breaking the application
   *  * @type {Promise<any>}
   * @private
   */
<% } %>
var __reject = function () {
  return new Promise(function (_, reject) {
    reject();
  });
}

window.tauri = {
  <% if (ctx.dev) { %>
    /**
     * @name invoke
     * @description Calls a Tauri Core feature, such as setTitle
     * @param {Object} args
     */
  <% } %>
  invoke: function invoke(args) {
    window.external.invoke(JSON.stringify(args));
  },

  <% if (ctx.dev) { %>
    /**
     * @name listen
     * @description Add an event listener to Tauri backend
     * @param {String} event
     * @param {Function} handler
     * @param {Boolean} once
     */
  <% } %>
  listen: function listen(event, handler) {
    <% if (tauri.whitelist.event === true || tauri.whitelist.all === true) { %>
    var once = arguments.length > 2 && arguments[2] !== undefined ? arguments[2] : false;
      this.invoke({
        cmd: 'listen',
        event: event,
        handler: window.tauri.transformCallback(handler, once),
        once: once
      });
    <% } else { %>
      <% if (ctx.dev) { %>
          return __whitelistWarning('event')
          <% } %>
        return __reject()
        <% } %>
  },

  <% if (ctx.dev) { %>
    /**
     * @name emit
     * @description Emits an evt to the Tauri back end
     * @param {String} evt
     * @param {Object} payload
     */
  <% } %>
  emit: function emit(evt, payload) {
    <% if (tauri.whitelist.event === true || tauri.whitelist.all === true) { %>
      this.invoke({
        cmd: 'emit',
        event: evt,
        payload: payload
      });
    <% } else { %>
      <% if (ctx.dev) { %>
          return __whitelistWarning('event')
          <% } %>
        return __reject()
        <% } %>
  },

  <% if (ctx.dev) { %>
    /**
     * @name transformCallback
     * @description Registers a callback with a uid
     * @param {Function} callback
     * @param {Boolean} once
     * @returns {*}
     */
  <% } %>
  transformCallback: function transformCallback(callback) {
    var once = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : false;
    var identifier = uid();

    window[identifier] = function (result) {
      if (once) {
        delete window[identifier];
      }

      return callback && callback(result);
    };

    return identifier;
  },

  <% if (ctx.dev) { %>
    /**
     * @name promisified
     * @description Turns a request into a chainable promise
     * @param {Object} args
     * @returns {Promise<any>}
     */
  <% } %>
  promisified: function promisified(args) {
    var _this = this;

    return new Promise(function (resolve, reject) {
      _this.invoke(_objectSpread({
        callback: _this.transformCallback(resolve),
        error: _this.transformCallback(reject)
      }, args));
    });
  },

  <% if (ctx.dev) { %>
    /**
     * @name readTextFile
     * @description Accesses a non-binary file on the user's filesystem
     * and returns the content. Permissions based on the app's PID owner
     * @param {String} path
     * @returns {*|Promise<any>|Promise}
     */
  <% } %>
  readTextFile: function readTextFile(path) {
    <% if (tauri.whitelist.readTextFile === true || tauri.whitelist.all === true) { %>
      return this.promisified({
        cmd: 'readTextFile',
        path: path
      });
    <% } else { %>
      <% if (ctx.dev) { %>
          return __whitelistWarning('readTextFile')
          <% } %>
        return __reject()
        <% } %>
  },

  <% if (ctx.dev) { %>
    /**
     * @name readBinaryFile
     * @description Accesses a binary file on the user's filesystem
     * and returns the content. Permissions based on the app's PID owner
     * @param {String} path
     * @returns {*|Promise<any>|Promise}
     */
  <% } %>
  readBinaryFile: function readBinaryFile(path) {
    <% if (tauri.whitelist.readBinaryFile === true || tauri.whitelist.all === true) { %>
      return this.promisified({
        cmd: 'readBinaryFile',
        path: path
      });
    <% } else { %>
      <% if (ctx.dev) { %>
          return __whitelistWarning('readBinaryFile')
          <% } %>
        return __reject()
        <% } %>
  },

  <% if (ctx.dev) { %>
    /**
     * @name writeFile
     * @description Write a file to the Local Filesystem.
     * Permissions based on the app's PID owner
     * @param {Object} cfg
     * @param {String} cfg.file
     * @param {String|Binary} cfg.contents
     */
  <% } %>
  writeFile: function writeFile(cfg) {
    <% if (tauri.whitelist.writeFile === true || tauri.whitelist.all === true) { %>
      if (_typeof(cfg) === 'object') {
        Object.freeze(cfg);
      }
      return this.promisified({
        cmd: 'writeFile',
        file: cfg.file,
        contents: cfg.contents
      });
    <% } else { %>
      <% if (ctx.dev) { %>
          return __whitelistWarning('writeFile')
          <% } %>
        return __reject()
        <% } %>
  },

  <% if (ctx.dev) { %>
    /**
     * @name readDir
     * @description Reads a directory
     * Permissions based on the app's PID owner
     * @param {String} path
     * @param {Object} [options]
     * @param {Boolean} [options.recursive]
     * @returns {*|Promise<any>|Promise}
     */
  <% } %>
  readDir: function readDir(path, options) {
    <% if (tauri.whitelist.readDir === true || tauri.whitelist.all === true) { %>
      return this.promisified({
        cmd: 'readDir',
        path: path,
        options: options
      });
    <% } else { %>
      <% if (ctx.dev) { %>
          return __whitelistWarning('readDir')
          <% } %>
        return __reject()
        <% } %>
  },

  <% if (ctx.dev) { %>
    /**
     * @name setTitle
     * @description Set the application's title
     * @param {String} title
     */
  <% } %>
  setTitle: function setTitle(title) {
    <% if (tauri.whitelist.setTitle === true || tauri.whitelist.all === true) { %>
      this.invoke({
        cmd: 'setTitle',
        title: title
      });
    <% } else { %>
      <% if (ctx.dev) { %>
    return __whitelistWarning('setTitle')
          <% } %>
    return __reject()
        <% } %>
  },

  <% if (ctx.dev) { %>
    /**
     * @name open
     * @description Open an URI
     * @param {String} uri
     */
  <% } %>
  open: function open(uri) {
    <% if (tauri.whitelist.open === true || tauri.whitelist.all === true) { %>
      this.invoke({
        cmd: 'open',
        uri: uri
      });
    <% } else { %>
    <% if (ctx.dev) { %>
      return __whitelistWarning('open')
          <% } %>
    return __reject()
        <% } %>
  },

  <% if (ctx.dev) { %>
    /**
     * @name execute
     * @description Execute a program with arguments.
     * Permissions based on the app's PID owner
     * @param {String} command
     * @param {String|Array} args
     * @returns {*|Promise<any>|Promise}
     */
  <% } %>
  execute: function execute(command, args) {
    <% if (tauri.whitelist.execute === true || tauri.whitelist.all === true) { %>

      if (_typeof(args) === 'object') {
        Object.freeze(args);
      }

      return this.promisified({
        cmd: 'execute',
        command: command,
        args: typeof args === 'string' ? [args] : args
      });
    <% } else { %>
      <% if (ctx.dev) { %>
        return __whitelistWarning('execute')
          <% } %>
        return __reject()
        <% } %>
  },

loadAsset: function loadAsset(assetName, assetType) {
  return this.promisified({
    cmd: 'loadAsset',
    asset: assetName,
    asset_type: assetType || 'unknown'
  })
}
};

// init tauri API
try {
  window.tauri.invoke({
    cmd: 'init'
  })
} catch (e) {
  window.addEventListener('DOMContentLoaded', function () {
    window.tauri.invoke({
      cmd: 'init'
    })
  }, true)
}

document.addEventListener('error', function (e) {
  var target = e.target
  while (target != null) {
    if (target.matches ? target.matches('img') : target.msMatchesSelector('img')) {
      window.tauri.loadAsset(target.src, 'image')
        .then(function (img) {
          target.src = img
        })
      break
    }
    target = target.parentElement
  }
}, true)

// open <a href="..."> links with the Tauri API
function __openLinks() {
  document.querySelector('body').addEventListener('click', function (e) {
    var target = e.target
    while (target != null) {
      if (target.matches ? target.matches('a') : target.msMatchesSelector('a')) {
        if (target.href && target.href.startsWith('http') && target.target === '_blank') {
          window.tauri.open(target.href)
          e.preventDefault()
        }
        break
      }
      target = target.parentElement
    }
  }, true)
}

if (document.readyState === 'complete' || document.readyState === 'interactive') {
  __openLinks()
} else {
  window.addEventListener('DOMContentLoaded', function () {
    __openLinks()
  }, true)
}
