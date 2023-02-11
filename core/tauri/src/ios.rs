use cocoa::base::{id, nil, NO, YES};
use objc::*;
use serde_json::Value as JsonValue;
use swift_rs::SRString;

use std::os::raw::{c_char, c_int};

type PluginMessageCallback = unsafe extern "C" fn(c_int, c_int, *const c_char);

extern "C" {
  pub fn post_ipc_message(
    webview: id,
    name: &SRString,
    method: &SRString,
    data: id,
    callback: usize,
    error: usize,
  );

  pub fn run_plugin_method(
    id: i32,
    name: &SRString,
    method: &SRString,
    data: id,
    callback: PluginMessageCallback,
  );

  pub fn on_webview_created(webview: id);
}

pub fn json_to_dictionary(json: JsonValue) -> id {
  if let serde_json::Value::Object(map) = json {
    unsafe {
      let dictionary: id = msg_send![class!(NSMutableDictionary), alloc];
      let data: id = msg_send![dictionary, init];
      for (key, value) in map {
        add_json_entry_to_dictionary(data, key, value);
      }
      data
    }
  } else {
    nil
  }
}

const UTF8_ENCODING: usize = 4;

struct NSString(id);

impl NSString {
  fn new(s: &str) -> Self {
    // Safety: objc runtime calls are unsafe
    NSString(unsafe {
      let ns_string: id = msg_send![class!(NSString), alloc];
      let ns_string: id = msg_send![ns_string,
                                            initWithBytes:s.as_ptr()
                                            length:s.len()
                                            encoding:UTF8_ENCODING];

      // The thing is allocated in rust, the thing must be set to autorelease in rust to relinquish control
      // or it can not be released correctly in OC runtime
      let _: () = msg_send![ns_string, autorelease];

      ns_string
    })
  }
}

unsafe fn add_json_value_to_array(array: id, value: JsonValue) {
  match value {
    JsonValue::Null => {
      let null: id = msg_send![class!(NSNull), null];
      let () = msg_send![array, addObject: null];
    }
    JsonValue::Bool(val) => {
      let value = if val { YES } else { NO };
      let v: id = msg_send![class!(NSNumber), numberWithBool: value];
      let () = msg_send![array, addObject: v];
    }
    JsonValue::Number(val) => {
      let number: id = if let Some(v) = val.as_i64() {
        msg_send![class!(NSNumber), numberWithInteger: v]
      } else if let Some(v) = val.as_u64() {
        msg_send![class!(NSNumber), numberWithUnsignedLongLong: v]
      } else if let Some(v) = val.as_f64() {
        msg_send![class!(NSNumber), numberWithDouble: v]
      } else {
        unreachable!()
      };
      let () = msg_send![array, addObject: number];
    }
    JsonValue::String(val) => {
      let () = msg_send![array, addObject: NSString::new(&val)];
    }
    JsonValue::Array(val) => {
      let nsarray: id = msg_send![class!(NSMutableArray), alloc];
      let inner_array: id = msg_send![nsarray, init];
      for value in val {
        add_json_value_to_array(inner_array, value);
      }
      let () = msg_send![array, addObject: inner_array];
    }
    JsonValue::Object(val) => {
      let dictionary: id = msg_send![class!(NSMutableDictionary), alloc];
      let data: id = msg_send![dictionary, init];
      for (key, value) in val {
        add_json_entry_to_dictionary(data, key, value);
      }
      let () = msg_send![array, addObject: data];
    }
  }
}

unsafe fn add_json_entry_to_dictionary(data: id, key: String, value: JsonValue) {
  let key = NSString::new(&key);
  match value {
    JsonValue::Null => {
      let null: id = msg_send![class!(NSNull), null];
      let () = msg_send![data, setObject:null forKey: key];
    }
    JsonValue::Bool(val) => {
      let value = if val { YES } else { NO };
      let () = msg_send![data, setObject:value forKey: key];
    }
    JsonValue::Number(val) => {
      let number: id = if let Some(v) = val.as_i64() {
        msg_send![class!(NSNumber), numberWithInteger: v]
      } else if let Some(v) = val.as_u64() {
        msg_send![class!(NSNumber), numberWithUnsignedLongLong: v]
      } else if let Some(v) = val.as_f64() {
        msg_send![class!(NSNumber), numberWithDouble: v]
      } else {
        unreachable!()
      };
      let () = msg_send![data, setObject:number forKey: key];
    }
    JsonValue::String(val) => {
      let () = msg_send![data, setObject:NSString::new(&val) forKey: key];
    }
    JsonValue::Array(val) => {
      let nsarray: id = msg_send![class!(NSMutableArray), alloc];
      let array: id = msg_send![nsarray, init];
      for value in val {
        add_json_value_to_array(array, value);
      }
      let () = msg_send![data, setObject:array forKey: key];
    }
    JsonValue::Object(val) => {
      let dictionary: id = msg_send![class!(NSMutableDictionary), alloc];
      let inner_data: id = msg_send![dictionary, init];
      for (key, value) in val {
        add_json_entry_to_dictionary(inner_data, key, value);
      }
      let () = msg_send![data, setObject:inner_data forKey: key];
    }
  }
}
