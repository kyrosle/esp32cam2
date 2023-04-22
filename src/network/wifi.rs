use anyhow::Result;
use embedded_svc::{
  ipv4,
  wifi::{AccessPointConfiguration, ClientConfiguration, Configuration, Wifi},
};
use esp_idf_hal::peripheral;
use esp_idf_svc::{
  eventloop::EspSystemEventLoop,
  netif::{EspNetif, EspNetifWait},
  nvs::EspDefaultNvsPartition,
  ping::EspPing,
  wifi::{EspWifi, WifiWait},
};
use std::{net::Ipv4Addr, time::Duration};

use crate::{PASS, SSID};

/// 返回一个Result对象，包含一个Box指向EspWifi对象的所有权
pub fn wifi(
  // 定义一个泛型函数，接受一个实现了Peripheral trait的modem对象
  modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
  // 定义一个系统事件循环对象
  sysloop: EspSystemEventLoop,
  // 定义一个可选的默认NVS分区对象
  default_nvs: Option<EspDefaultNvsPartition>,
) -> Result<Box<EspWifi<'static>>> {
  println!("hello -----------------------------> wifi?");
  // 创建一个EspWifi对象，并将其放入Box中
  let mut wifi = Box::new(EspWifi::new(modem, sysloop.clone(), default_nvs)?);

  println!("Wifi created, about to scan...");

  // 扫描可用的WiFi接入点，并将结果存储在ap_infos变量中
  let ap_infos = wifi.scan()?;
  // 将ap_infos中的每个接入点的SSID存储在一个Vec中
  let list = ap_infos.iter().map(|a| a.ssid.clone()).collect::<Vec<_>>();

  // 打印当前存在的SSID
  for a in list {
    println!("{}", a);
  }
  // 查找配置的接入点的SSID，并将结果存储在ours变量中
  let ours = ap_infos.into_iter().find(|a| a.ssid == SSID);
  // 如果找到了配置的接入点
  let channel = if let Some(ours) = ours {
    // 返回接入点的信道号
    println!(
      "Found configured access point {} on channel {}",
      SSID, ours.channel
    );
    Some(ours.channel)
  } else {
    // 如果没有找到配置的接入点
    println!(
      "Configured access point {} not found during scanning, will go with unknown channel",
      SSID
    );
    None
  };

  wifi.set_configuration(&Configuration::Mixed(
    // 配置WiFi连接
    ClientConfiguration {
      // 客户端配置
      ssid: SSID.into(),     // 接入点的SSID
      password: PASS.into(), // 接入点的密码
      channel,               // 接入点的信道号
      ..Default::default()   // 使用默认值的其他配置
    },
    AccessPointConfiguration {
      // 接入点配置
      ssid: "aptest".into(),         // 接入点的SSID
      channel: channel.unwrap_or(1), // 接入点的信道号
      ..Default::default()           // 使用默认值的其他配置
    },
  ))?;
  // 启动WiFi连接
  wifi.start()?;

  println!("Starting wifi...");

  // 等待WiFi连接启动
  if !WifiWait::new(&sysloop)?
    .wait_with_timeout(Duration::from_secs(20), || wifi.is_started().unwrap())
  {
    // 如果WiFi连接没有启动，则返回一个错误
    anyhow::bail!("Wifi did not start");
  }

  println!("Connecting wifi...");

  // 连接WiFi接入点
  wifi.connect()?;

  // 等待WiFi连接成功
  if !EspNetifWait::new::<EspNetif>(wifi.sta_netif(), &sysloop)?.wait_with_timeout(
    Duration::from_secs(20),
    || {
      wifi.is_connected().unwrap()
        && wifi.sta_netif().get_ip_info().unwrap().ip != Ipv4Addr::new(0, 0, 0, 0)
    },
  ) {
    // 如果WiFi连接失败，则返回一个错误
    anyhow::bail!("Wifi did not connect or did not receive a DHCP lease");
  }
  // 获取WiFi接口的IP信息
  let ip_info = wifi.sta_netif().get_ip_info()?;

  // 使用ping函数测试网关是否可达
  ping(ip_info.subnet.gateway)?;

  Ok(wifi)
}

/// 定义了一个名为ping的函数，用于测试网关是否可达.
fn ping(ip: ipv4::Ipv4Addr) -> Result<()> {
  println!("About to do some pings for {:?}", ip);
  // 使用EspPing对象执行ping操作，并将结果存储在ping_summary变量中
  let ping_summary = EspPing::default().ping(ip, &Default::default())?;
  // 如果发送的ping包数量不等于接收的ping包数量
  if ping_summary.transmitted != ping_summary.received {
    anyhow::bail!("Pinging gateway {} resulted in timeouts", ip);
  }

  println!("Pinging done");

  Ok(())
}
