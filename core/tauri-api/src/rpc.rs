use serde::Serialize;
use serde_json::Value as JsonValue;

/// The information about this is quite limited. On Chrome/Edge and Firefox, [the maximum string size is approximately 1 GB](https://stackoverflow.com/a/34958490).
///
/// [From MDN:](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/length#description)
///
/// ECMAScript 2016 (ed. 7) established a maximum length of 2^53 - 1 elements. Previously, no maximum length was specified.
///
/// In Firefox, strings have a maximum length of 2\*\*30 - 2 (~1GB). In versions prior to Firefox 65, the maximum length was 2\*\*28 - 1 (~256MB).
pub const MAX_JSON_STR_LEN: usize = usize::pow(2, 30) - 2;

/// Safely transforms & escapes a JSON String -> JSON.parse('{json}')
//  Single quotes are the fastest string for the JavaScript engine to build.
//  Directly transforming the string byte-by-byte is faster than a double String::replace()
pub fn escape_json_parse(mut json: String) -> String {
  const BACKSLASH_BYTE: u8 = b'\\';
  const SINGLE_QUOTE_BYTE: u8 = b'\'';

  // Safety:
  //
  // Directly mutating the bytes of a String is considered unsafe because you could end
  // up inserting invalid UTF-8 into the String.
  //
  // In this case, we are working with single-byte \ (backslash) and ' (single quotes),
  // and only INSERTING a backslash in the position proceeding it, which is safe to do.
  //
  // Note the debug assertion that checks whether the String is valid UTF-8.
  // In the test below this assertion will fail if the emojis in the test strings cause problems.

  let bytes: &mut Vec<u8> = unsafe { json.as_mut_vec() };
  let mut i = 0;
  while i < bytes.len() {
    let byte = bytes[i];
    if matches!(byte, BACKSLASH_BYTE | SINGLE_QUOTE_BYTE) {
      bytes.insert(i, BACKSLASH_BYTE);
      i += 1;
    }
    i += 1;
  }

  debug_assert!(String::from_utf8(bytes.to_vec()).is_ok());

  format!("JSON.parse('{}')", json)
}

#[test]
fn test_escape_json_parse() {
  let dangerous_json = String::from(
    r#"{"test":"don\\🚀🐱‍👤\\'t forget to escape me!🚀🐱‍👤","te🚀🐱‍👤st2":"don't forget to escape me!","test3":"\\🚀🐱‍👤\\\\'''\\\\🚀🐱‍👤\\\\🚀🐱‍👤\\'''''"}"#,
  );

  let definitely_escaped_dangerous_json = format!(
    "JSON.parse('{}')",
    dangerous_json.replace('\\', "\\\\").replace('\'', "\\'")
  );
  let escape_single_quoted_json_test = escape_json_parse(dangerous_json);

  let result = r#"JSON.parse('{"test":"don\\\\🚀🐱‍👤\\\\\'t forget to escape me!🚀🐱‍👤","te🚀🐱‍👤st2":"don\'t forget to escape me!","test3":"\\\\🚀🐱‍👤\\\\\\\\\'\'\'\\\\\\\\🚀🐱‍👤\\\\\\\\🚀🐱‍👤\\\\\'\'\'\'\'"}')"#;
  assert_eq!(definitely_escaped_dangerous_json, result);
  assert_eq!(escape_single_quoted_json_test, result);
}

/// Formats a function name and argument to be evaluated as callback.
///
/// This will serialize primitive JSON types (e.g. booleans, strings, numbers, etc.) as JavaScript literals,
/// but will serialize arrays and objects whose serialized JSON string is smaller than 1 GB as `JSON.parse('...')`
/// https://github.com/GoogleChromeLabs/json-parse-benchmark
///
/// # Examples
/// ```
/// use tauri_api::rpc::format_callback;
/// // callback with a string argument
/// let cb = format_callback("callback-function-name", "the string response");
/// assert!(cb.contains(r#"window["callback-function-name"]("the string response")"#));
/// ```
///
/// ```
/// use tauri_api::rpc::format_callback;
/// use serde::Serialize;
/// // callback with JSON argument
/// #[derive(Serialize)]
/// struct MyResponse {
///   value: String
/// }
/// let cb = format_callback("callback-function-name", serde_json::to_value(&MyResponse {
///   value: "some value".to_string()
/// }).expect("failed to serialize"));
/// assert!(cb.contains(r#"window["callback-function-name"](JSON.parse('{"value":"some value"}'))"#));
/// ```
pub fn format_callback<T: Into<JsonValue>, S: AsRef<str>>(function_name: S, arg: T) -> String {
  macro_rules! format_callback {
    ( $arg:expr ) => {
      format!(
        r#"
          if (window["{fn}"]) {{
            window["{fn}"]({arg})
          }} else {{
            console.warn("[TAURI] Couldn't find callback id {fn} in window. This happens when the app is reloaded while Rust is running an asynchronous operation.")
          }}
        "#,
        fn = function_name.as_ref(),
        arg = $arg
      )
    }
  }

  let json_value = arg.into();

  // We should only use JSON.parse('{arg}') if it's an array or object.
  // We likely won't get any performance benefit from other data types.
  if matches!(json_value, JsonValue::Array(_) | JsonValue::Object(_)) {
    let as_str = json_value.to_string();

    // Explicitly drop json_value to avoid storing both the Rust "JSON" and serialized String JSON in memory twice, as <T: Display>.tostring() takes a reference.
    drop(json_value);

    format_callback!(if as_str.len() < MAX_JSON_STR_LEN {
      escape_json_parse(as_str)
    } else {
      as_str
    })
  } else {
    format_callback!(json_value)
  }
}

/// Formats a Result type to its Promise response.
/// Useful for Promises handling.
/// If the Result `is_ok()`, the callback will be the `success_callback` function name and the argument will be the Ok value.
/// If the Result `is_err()`, the callback will be the `error_callback` function name and the argument will be the Err value.
///
/// * `result` the Result to check
/// * `success_callback` the function name of the Ok callback. Usually the `resolve` of the JS Promise.
/// * `error_callback` the function name of the Err callback. Usually the `reject` of the JS Promise.
///
/// Note that the callback strings are automatically generated by the `invoke` helper.
///
/// # Examples
/// ```
/// use tauri_api::rpc::format_callback_result;
/// let res: Result<u8, &str> = Ok(5);
/// let cb = format_callback_result(res, "success_cb", "error_cb").expect("failed to format");
/// assert!(cb.contains(r#"window["success_cb"](5)"#));
///
/// let res: Result<&str, &str> = Err("error message here");
/// let cb = format_callback_result(res, "success_cb", "error_cb").expect("failed to format");
/// assert!(cb.contains(r#"window["error_cb"]("error message here")"#));
/// ```
pub fn format_callback_result<T: Serialize, E: Serialize>(
  result: Result<T, E>,
  success_callback: impl AsRef<str>,
  error_callback: impl AsRef<str>,
) -> crate::Result<String> {
  let rpc = match result {
    Ok(res) => format_callback(success_callback, serde_json::to_value(res)?),
    Err(err) => format_callback(error_callback, serde_json::to_value(err)?),
  };
  Ok(rpc)
}

#[cfg(test)]
mod test {
  use crate::rpc::*;
  use quickcheck_macros::quickcheck;

  // check abritrary strings in the format callback function
  #[quickcheck]
  fn qc_formating(f: String, a: String) -> bool {
    // can not accept empty strings
    if !f.is_empty() && !a.is_empty() {
      // call format callback
      let fc = format_callback(f.clone(), a.clone());
      fc.contains(&format!(
        r#"window["{}"](JSON.parse('{}'))"#,
        f,
        serde_json::Value::String(a.clone()),
      )) || fc.contains(&format!(
        r#"window["{}"]({})"#,
        f,
        serde_json::Value::String(a),
      ))
    } else {
      true
    }
  }

  // check arbitrary strings in format_callback_result
  #[quickcheck]
  fn qc_format_res(result: Result<String, String>, c: String, ec: String) -> bool {
    let resp = format_callback_result(result.clone(), c.clone(), ec.clone())
      .expect("failed to format callback result");
    let (function, value) = match result {
      Ok(v) => (c, v),
      Err(e) => (ec, e),
    };

    resp.contains(&format!(
      r#"window["{}"]({})"#,
      function,
      serde_json::Value::String(value),
    ))
  }
}
