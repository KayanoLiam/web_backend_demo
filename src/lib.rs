/// Actix-Web学习示例库
///
/// 这个库包含了使用Actix-Web框架的各种示例，
/// 包括路由处理、错误处理、请求参数提取等功能。
///
/// # 模块
/// * `models` - 数据模型和结构体
/// * `handlers` - HTTP请求处理函数
/// * `errors` - 自定义错误类型和实现
/// * `config` - 应用配置函数
/// * `utils` - 工具函数

// 导出所有模块，使它们可以被其他模块引用
pub mod models;    // 数据模型和结构体
pub mod handlers;  // HTTP请求处理函数
pub mod errors;    // 自定义错误类型和实现
pub mod config;    // 应用配置函数
pub mod utils;     // 工具函数
