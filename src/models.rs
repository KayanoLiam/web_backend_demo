// 标准库导入
use std::sync::Mutex;  // 用于线程安全的共享状态

// 外部库导入
use serde::{Deserialize, Serialize};  // 用于JSON序列化和反序列化
use actix_web::{body::BoxBody, HttpResponse, Responder};  // 用于HTTP响应处理

/// 应用状态结构体
///
/// 用于在整个应用程序中共享应用名称
/// 通过依赖注入传递给处理函数
pub struct AppState {
    pub app_name: String,  // 应用程序名称
}

/// 带计数器的应用状态结构体
///
/// 用于在请求之间共享和修改计数器值
/// 使用Mutex确保线程安全
pub struct AppStateWithCounter {
    pub counter: Mutex<i32>,  // 线程安全的整数计数器
}

/// 路径参数结构体
///
/// 用于从URL路径中提取用户ID和名称
/// 例如：/path2/123/alice 会提取 user_id=123, name="alice"
#[derive(Deserialize)]  // 启用从路径参数到结构体的自动反序列化
pub struct UserInfo {
    pub user_id: u32,    // 用户ID，无符号32位整数
    pub name: String,    // 用户名称
}

/// 查询参数结构体
///
/// 用于从URL查询字符串中提取搜索参数
/// 例如：/query?q=rust&lang=en 会提取 q="rust", lang=Some("en")
#[derive(Deserialize)]  // 启用从查询参数到结构体的自动反序列化
pub struct SearchQuery {
    pub q: String,                // 必需的查询字符串
    pub lang: Option<String>,     // 可选的语言参数
}

/// JSON输入结构体
///
/// 用于从请求体中提取JSON数据
/// 例如：{"username": "alice", "email": "alice@example.com"}
#[derive(Deserialize)]  // 启用从JSON到结构体的自动反序列化
pub struct UserIput {
    pub username: String,  // 用户名
    pub email: String,     // 电子邮件
}

/// 表单输入结构体
///
/// 用于从表单提交中提取用户登录信息
/// 例如：username=alice&password=secret
#[derive(Deserialize)]  // 启用从表单数据到结构体的自动反序列化
pub struct LoginInfo {
    pub username: String,  // 用户名
    pub password: String,  // 密码
}

/// 响应结构体
///
/// 用于生成JSON响应
/// 包含用户名和年龄信息
#[derive(Serialize)]  // 启用结构体到JSON的自动序列化
pub struct MyStruct {
    pub name: String,  // 用户名
    pub age: u32,      // 年龄，无符号32位整数
}

/// 为MyStruct实现Responder trait
///
/// 使MyStruct可以直接作为处理函数的返回值
/// 自动转换为HTTP响应
impl Responder for MyStruct {
    // 指定响应体的类型为 BoxBody，这是 actix-web 推荐的响应体类型
    type Body = BoxBody;

    /// 实现 respond_to 方法，将 MyStruct 转换为 HTTP 响应
    ///
    /// # 参数
    /// * `_req` - HTTP请求引用，本实现中未使用
    ///
    /// # 返回值
    /// * 返回包含JSON数据的HTTP 200 OK响应
    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        // 将结构体序列化为 JSON 字符串
        // unwrap()在序列化失败时会导致panic，生产环境应处理错误
        let body = serde_json::to_string(&self).unwrap();

        // 构建 HTTP 响应
        // 1. 设置状态码为200 OK
        // 2. 设置 Content-Type 为 application/json
        // 3. 将序列化后的 JSON 作为响应体
        HttpResponse::Ok()
            .content_type("application/json")
            .body(body)
    }
}
