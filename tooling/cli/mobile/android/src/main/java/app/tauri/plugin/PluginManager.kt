package app.tauri.plugin

import android.webkit.WebView
import app.tauri.Logger

class PluginManager(private val webView: WebView) {
  private val plugins: HashMap<String, PluginHandle> = HashMap()

  fun load(name: String, plugin: Plugin) {
    plugin.load(webView)
    plugins[name] = PluginHandle(plugin)
  }

  fun postMessage(pluginId: String, methodName: String, data: JSObject, callback: Long, error: Long) {
    Logger.verbose(
      Logger.tags("Plugin"),
      "Tauri plugin: pluginId: $pluginId, methodName: $methodName, callback: $callback, error: $error"
    )

    val invoke = Invoke({ successResult, errorResult ->
      val (fn, result) = if (errorResult == null) Pair(callback, successResult) else Pair(
        error,
        errorResult
      )
      webView.evaluateJavascript("window['_$fn']($result)", null)
    }, data)
    try {
      val plugin = plugins[pluginId]
      if (plugin == null) {
        invoke.reject("Plugin $pluginId not initialized")
      } else {
        plugins[pluginId]?.invoke(methodName, invoke)
      }
    } catch (e: Exception) {
      invoke.reject(e.toString())
    }
  }
}
