/// 摄像机初始化
pub fn camera_init() -> bool {
  /*
  这段代码是用于配置 ESP32-CAM 模块中 OV2640 摄像头的参数。具体来说，它定义了一个名为 camera_config 的结构体，
  包含了 OV2640 摄像头的各种参数，例如引脚配置、像素格式、帧大小、JPEG 压缩质量等。

  下面是对 camera_config 结构体中各个参数的解释：

  pin_pwdn：摄像头的电源控制引脚，用于控制摄像头的开关机。在这里，它被配置为引脚 32。
  pin_reset：摄像头的复位引脚，用于复位摄像头。在这里，它被配置为 -1，表示不使用复位引脚。
  pin_xclk：摄像头的时钟引脚，用于控制摄像头的时序。在这里，它被配置为引脚 0。
  pin_sscb_sda：摄像头的 SCCB 数据引脚，用于与摄像头进行通信。在这里，它被配置为引脚 26。
  pin_sscb_scl：摄像头的 SCCB 时钟引脚，用于与摄像头进行通信。在这里，它被配置为引脚 27。
  pin_d7 ~ pin_d0：摄像头的数据引脚，用于传输图像数据。在这里，它们被配置为引脚 35、34、39、36、21、19、18 和 5。
  pin_vsync：摄像头的垂直同步引脚，用于同步图像数据。在这里，它被配置为引脚 25。
  pin_href：摄像头的行同步引脚，用于同步图像数据。在这里，它被配置为引脚 23。
  pin_pclk：摄像头的像素时钟引脚，用于控制像素数据的传输速率。在这里，它被配置为引脚 22。
  xclk_freq_hz：摄像头的时钟频率，用于控制摄像头的帧率。在这里，它被配置为 20MHz。
  ledc_timer：LED 控制器的定时器编号，用于控制摄像头的 LED 灯。在这里，它被配置为 LEDC_TIMER_0。
  ledc_channel：LED 控制器的通道编号，用于控制摄像头的 LED 灯。在这里，它被配置为 LEDC_CHANNEL_0。
  pixel_format：摄像头的像素格式，用于指定图像数据的编码方式。在这里，它被配置为 JPEG 格式。
  frame_size：摄像头的帧大小，用于指定图像的分辨率。在这里，它被配置为 QVGA 分辨率。
  jpeg_quality：JPEG 压缩质量，用于控制 JPEG 图像的压缩比例。在这里，它被配置为 16。
  fb_count：帧缓冲区的数量，用于控制帧缓冲区的个数。在这里，它被配置为 1。
  fb_location：帧缓冲区的存储位置，用于指定帧缓冲区的存储位置。在这里，它被配置为 PSRAM 存储器。
  grab_mode：帧缓冲区的获取模式，用于指定帧缓冲区的获取方式。在这里，它被配置为当帧缓冲区为空
  */
  let camera_config = esp_idf_sys::camera_config_t {
    pin_pwdn: 32,
    pin_reset: -1,
    pin_xclk: 0,
    __bindgen_anon_1: esp_idf_sys::camera_config_t__bindgen_ty_1 { pin_sscb_sda: 26 },
    __bindgen_anon_2: esp_idf_sys::camera_config_t__bindgen_ty_2 { pin_sscb_scl: 27 },
    pin_d7: 35,
    pin_d6: 34,
    pin_d5: 39,
    pin_d4: 36,
    pin_d3: 21,
    pin_d2: 19,
    pin_d1: 18,
    pin_d0: 5,
    pin_vsync: 25,
    pin_href: 23,
    pin_pclk: 22,

    //XCLK 20MHz or 10MHz for OV2640 double FPS (Experimental)
    xclk_freq_hz: 20000000,
    ledc_timer: esp_idf_sys::ledc_timer_t_LEDC_TIMER_0,
    ledc_channel: esp_idf_sys::ledc_channel_t_LEDC_CHANNEL_0,

    pixel_format: esp_idf_sys::pixformat_t_PIXFORMAT_JPEG, //YUV422,GRAYSCALE,RGB565,JPEG
    frame_size: esp_idf_sys::framesize_t_FRAMESIZE_QVGA, //QQVGA-UXGA Do not use sizes above QVGA when not JPEG

    jpeg_quality: 16, //0-63 lower number means higher quality
    fb_count: 1,      //if more than one, i2s runs in continuous mode. Use only with JPEG
    fb_location: esp_idf_sys::camera_fb_location_t_CAMERA_FB_IN_PSRAM,
    grab_mode: esp_idf_sys::camera_grab_mode_t_CAMERA_GRAB_WHEN_EMPTY,

    sccb_i2c_port: 0,
  };

  // 载入摄像机配置
  if unsafe { esp_idf_sys::esp_camera_init(&camera_config) } != 0 {
    println!("camera init failed!");
    false
  } else {
    println!("camera ready! >>>>>>>>>>>>>");
    true
  }
}
