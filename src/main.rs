// use std::{ptr::copy_nonoverlapping, sync::Arc};

use core::ptr;
use std::{ffi::CString, thread, time::Duration};

use anyhow::Result;
// use camera::camera_init;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_sys as _;
use network::wifi;

use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};

use crate::{
  camera::camera_init,
  network::{default_configuration, jpg_stream_httpd_handler},
};
// use test::{test_fs, test_tcp};

pub mod camera;
pub mod network;
// pub mod test;

/*
SSID: wifi名称
PASS: wifi密码
*/
pub const SSID: &str = r#"WEACSOFT"#;
pub const PASS: &str = r#"#weacsoft#417"#;

fn main() -> Result<()> {
  esp_idf_sys::link_patches(); // 链接ESP-IDF的补丁

  let peripherals = Peripherals::take().unwrap(); // 获取外设对象

  /* camera initialize */
  camera_init();

  /* wifi initialize */
  // 获取系统事件循环对象
  let sysloop = EspSystemEventLoop::take()?;
  // 获取默认NVS分区对象
  let default_nvs = EspDefaultNvsPartition::take().ok();
  // 初始化WiFi连接
  let mut _wifi = wifi(peripherals.modem, sysloop, default_nvs)?;

  /* webserver */
  // Entry: https://ip/stream.jpg
  // 创建一个C字符串
  let c_str = CString::new("/stream.jpg").unwrap();
  let uri_handler_jpg: esp_idf_sys::httpd_uri_t = esp_idf_sys::httpd_uri_t {
    // 定义一个httpd_uri_t对象
    uri: c_str.as_ptr(),                       // URI路径
    method: esp_idf_sys::http_method_HTTP_GET, // HTTP方法
    handler: Some(jpg_stream_httpd_handler),   // 处理函数
    user_ctx: ptr::null_mut(),                 // 用户上下文
  };
  // 定义一个httpd_handle_t对象
  let mut server: esp_idf_sys::httpd_handle_t = ptr::null_mut();
  // 获取server的可变引用
  let server_ref = &mut server;
  // 创建一个httpd_config_t对象
  let config: esp_idf_sys::httpd_config_t = default_configuration(80, 443);
  println!("{:?}", config);
  // 启动HTTP服务器
  let status = unsafe { esp_idf_sys::httpd_start(server_ref, &config) };
  println!("{}--{:?}", status, server);
  // 注册URI处理函数
  unsafe { esp_idf_sys::httpd_register_uri_handler(server, &uri_handler_jpg) };

  loop {
    // 进入无限循环
    thread::sleep(Duration::from_secs(2000)); // 线程休眠2秒
  }
}
