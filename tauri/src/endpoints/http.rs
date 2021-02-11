use crate::ApplicationDispatcherExt;
use tauri_api::http::{make_request as request, HttpRequestOptions};

/// Makes an HTTP request and resolves the response to the webview
pub async fn make_request<D: ApplicationDispatcherExt>(
  webview_manager: &crate::WebviewManager<D>,
  options: HttpRequestOptions,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview_manager,
    async move { request(options).map_err(|e| e.into()) },
    callback,
    error,
  )
  .await;
}
