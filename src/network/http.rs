use std::{
  ffi::CString,
  sync::{Arc, Condvar, Mutex},
};

use anyhow::{bail, Result};
use core::ptr;
use embedded_svc::{
  http::{client::Client as HttpClient, Method, Status},
  httpd::{registry::Registry, Response},
  io::Write,
  utils::io,
};
use esp_idf_svc::{
  http::client::{Configuration as HttpConfiguration, EspHttpConnection},
  httpd as idf,
};

/// 这段代码是用于配置ESP32的HTTP服务器的默认参数。其中，http_port和https_port分别表示HTTP和HTTPS的端口号。
/// httpd_config_t是一个结构体，包含了多个参数，如任务优先级、堆栈大小、服务器端口号、最大连接数等。
/// 这些参数可以根据实际需求进行修改。
/// 例如，如果启用了HTTPS，堆栈大小会增加到10240，最大连接数会减少到4。
/// 此外，还可以设置全局用户上下文和传输上下文，以及打开、关闭和URI匹配函数等回调函数。
/// 最后，还可以设置是否启用SO_LINGER选项和超时时间等。
#[allow(unused)]
pub fn default_configuration(http_port: u16, https_port: u16) -> esp_idf_sys::httpd_config_t {
  esp_idf_sys::httpd_config_t {
    task_priority: 5,
    stack_size: if https_port != 0 { 10240 } else { 4096 },
    core_id: std::i32::MAX,
    server_port: http_port,
    ctrl_port: 32768,
    max_open_sockets: if https_port != 0 { 4 } else { 7 },
    max_uri_handlers: 8,
    max_resp_headers: 8,
    backlog_conn: 5,
    lru_purge_enable: https_port != 0,
    recv_wait_timeout: 5,
    send_wait_timeout: 5,
    global_user_ctx: ptr::null_mut(),
    global_user_ctx_free_fn: None,
    global_transport_ctx: ptr::null_mut(),
    global_transport_ctx_free_fn: None,
    open_fn: None,
    close_fn: None,
    uri_match_fn: None,
    enable_so_linger: true,
    linger_timeout: 10,
  }
}

pub fn build_client() -> Result<HttpClient<EspHttpConnection>> {
  Ok(HttpClient::wrap(EspHttpConnection::new(
    &HttpConfiguration {
      crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
      ..Default::default()
    },
  )?))
}

/// Send a HTTP GET request.
#[allow(unused)]
pub fn get_request(client: &mut HttpClient<EspHttpConnection>) -> Result<()> {
  // Prepare headers and URL
  let headers = [("accept", "text/plain"), ("connection", "close")];
  let url = "http://ifconfig.net/";

  // Send request
  //
  // Note: If you don't want to pass in any headers, you can also use `client.get(url, headers)`.
  let request = client.request(Method::Get, url, &headers)?;
  println!("-> GET {}", url);
  let mut response = request.submit()?;

  // Process response
  let status = response.status();
  println!("<- {}", status);
  println!();
  let (_headers, mut body) = response.split();
  let mut buf = [0u8; 1024];
  let bytes_read = io::try_read_full(&mut body, &mut buf).map_err(|e| e.0)?;
  println!("Read {} bytes", bytes_read);
  match std::str::from_utf8(&buf[0..bytes_read]) {
    Ok(body_string) => println!(
      "Response body (truncated to {} bytes): {:?}",
      buf.len(),
      body_string
    ),
    Err(e) => eprintln!("Error decoding response body: {}", e),
  };

  // Drain the remaining response bytes
  while body.read(&mut buf)? > 0 {}

  Ok(())
}

/// Send a HTTP POST request.
#[allow(unused)]
pub fn post_request(client: &mut HttpClient<EspHttpConnection>) -> Result<()> {
  // Prepare payload
  let payload = b"Hello world!";

  // Prepare headers and URL
  let content_length_header = format!("{}", payload.len());
  let headers = [
    ("accept", "text/plain"),
    ("content-type", "text/plain"),
    ("connection", "close"),
    ("content-length", &*content_length_header),
  ];
  let url = "http://example.org/";

  // Send request
  let mut request = client.post(url, &headers)?;
  request.write_all(payload)?;
  request.flush()?;
  println!("-> POST {}", url);
  let mut response = request.submit()?;

  // Process response
  let status = response.status();
  println!("<- {}", status);
  println!();
  let (_headers, mut body) = response.split();
  let mut buf = [0u8; 1024];
  let bytes_read = io::try_read_full(&mut body, &mut buf).map_err(|e| e.0)?;
  println!("Read {} bytes", bytes_read);
  match std::str::from_utf8(&buf[0..bytes_read]) {
    Ok(body_string) => println!(
      "Response body (truncated to {} bytes): {:?}",
      buf.len(),
      body_string
    ),
    Err(e) => eprintln!("Error decoding response body: {}", e),
  };

  // Drain the remaining response bytes
  while body.read(&mut buf)? > 0 {}

  Ok(())
}

pub fn httpd(mutex: Arc<(Mutex<Option<u32>>, Condvar)>) -> Result<idf::Server> {
  let server = idf::ServerRegistry::new()
    .at("/")
    .get(|_| Ok("Hello from Rust!".into()))?
    .at("/foo")
    .get(|_| bail!("Boo, something happened!"))?
    .at("/bar")
    .get(|_| {
      Response::new(403)
        .status_message("No permissions")
        .body("You have no permission to access this page".into())
        .into()
    })?
    .at("/panic")
    .get(|_| panic!("User requested a panic!"))?;

  server.start(&Default::default())
}

/// 这段代码是用于实现ESP32-CAM的JPEG流式传输功能的HTTP请求处理函数。
/// 在函数中，首先定义了流式传输的内容类型、分界线和JPEG图片的格式。
/// 然后，通过esp_camera_fb_get()函数获取摄像头拍摄的JPEG图片，并将其发送给HTTP客户端。
/// 在发送图片之前，需要先发送分界线和图片的格式信息。
/// 最后，通过esp_camera_fb_return()函数释放图片缓存。这个函数是一个无限循环，可以不断地获取和发送图片，实现实时的JPEG流式传输。
pub unsafe extern "C" fn jpg_stream_httpd_handler(
  r: *mut esp_idf_sys::httpd_req_t,
) -> esp_idf_sys::esp_err_t {
  // 定义流式传输的内容类型、分界线和JPEG图片的格式
  let _stream_content_type: CString =
    CString::new("multipart/x-mixed-replace;boundary=123456789000000000000987654321").unwrap();
  let _stream_boundary: CString = CString::new("\r\n--123456789000000000000987654321\r\n").unwrap();
  let _stream_part: CString =
    CString::new("Content-Type: image/jpeg\r\nContent-Length: %u\r\n\r\n").unwrap();

  let part_buf = [0; 64];
  // 设置HTTP响应的内容类型为流式传输
  esp_idf_sys::httpd_resp_set_type(r, _stream_content_type.as_ptr());

  loop {
    println!("jpg_stream_httpd_handler !!!!");
    // 获取摄像头拍摄的JPEG图片
    let fb = esp_idf_sys::esp_camera_fb_get();
    println!("Picture taken! Its size was: {} bytes", unsafe {
      (*fb).len
    });

    // 发送分界线和图片的格式信息
    esp_idf_sys::httpd_resp_send_chunk(
      r,
      _stream_boundary.as_ptr(),
      esp_idf_sys::strlen(_stream_boundary.as_ptr()) as isize,
    );

    let hlen = esp_idf_sys::snprintf(
      part_buf.as_ptr() as *mut i8,
      64,
      _stream_part.as_ptr(),
      (*fb).len,
    );
    // 发送图片的格式信息
    esp_idf_sys::httpd_resp_send_chunk(r, part_buf.as_ptr() as *mut i8, hlen as isize);
    // 发送JPEG图片数据
    esp_idf_sys::httpd_resp_send_chunk(r, (*fb).buf as *mut i8, (*fb).len as isize);
    // 释放图片缓存
    esp_idf_sys::esp_camera_fb_return(fb);
  }
}
