// 外部库导入
use actix_web::{error, http, HttpResponse, ResponseError};  // 用于错误处理和HTTP响应
use actix_web::body::BoxBody;  // 用于HTTP响应体
use derive_more::{Display, Error};  // 用于自动派生Display和Error trait

/// 简单错误结构体
///
/// 最基本的错误类型，只包含一个错误消息
/// 使用默认的ResponseError实现，返回500 Internal Server Error
#[derive(Debug, Display, Error)]  // 自动派生Debug、Display和Error trait
pub struct MyError {
    pub name: &'static str,  // 错误消息，使用静态生命周期字符串
}

/// 为MyError实现ResponseError trait
///
/// 使用默认实现，不自定义错误响应
/// 默认返回500 Internal Server Error状态码
impl error::ResponseError for MyError {}

/// 枚举错误类型
///
/// 使用枚举表示不同类型的错误
/// 每种错误类型对应不同的HTTP状态码
#[derive(Debug, Display, Error)]  // 自动派生Debug、Display和Error trait
pub enum MyNewError {
    #[display(fmt = "内部错误")]  // 定义Display输出格式
    InternalError,                // 内部服务器错误

    #[display(fmt = "请求超时")]
    Timeout,                      // 请求超时错误

    #[display(fmt = "请求错误")]
    BadClientData,                // 客户端数据错误
}

/// 为MyNewError实现ResponseError trait
///
/// 自定义错误响应和状态码
impl error::ResponseError for MyNewError {
    /// 定义当错误发生时如何生成HTTP响应
    ///
    /// # 返回值
    /// * 返回包含错误信息的HTTP响应
    fn error_response(&self) -> HttpResponse<BoxBody> {
        // 使用self.status_code()获取对应的HTTP状态码，构建响应
        HttpResponse::build(self.status_code())
            // 设置响应头Content-Type为application/json
            .content_type("application/json")
            // 响应体内容为错误的字符串描述（即枚举的Display实现）
            .body(self.to_string())
    }

    /// 定义每种错误对应的HTTP状态码
    ///
    /// # 返回值
    /// * 返回对应错误类型的HTTP状态码
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            // InternalError映射为500 Internal Server Error
            MyNewError::InternalError => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            // Timeout映射为408 Request Timeout
            MyNewError::Timeout => actix_web::http::StatusCode::REQUEST_TIMEOUT,
            // BadClientData映射为400 Bad Request
            MyNewError::BadClientData => actix_web::http::StatusCode::BAD_REQUEST,
        }
    }
}

/// 简单错误结构体
///
/// 用于演示如何将自定义错误映射为actix-web错误
/// 不直接实现ResponseError，而是通过map_err转换
#[derive(Debug)]  // 仅派生Debug trait
pub struct MySimpleError {
    pub name: &'static str,  // 错误消息，使用静态生命周期字符串
}

/// 用户错误枚举
///
/// 用于表示与用户输入相关的错误
/// 目前只包含验证错误类型
#[derive(Debug, Display, Error)]  // 自动派生Debug、Display和Error trait
pub enum UserError {
    #[display(fmt = "验证错误: {field}")]  // 定义Display输出格式，包含字段名
    /// 验证错误变体
    ///
    /// 包含一个field字段，指示哪个字段验证失败
    /// 这里的花括号用于定义结构体变体（struct variant）
    ValidationError { field: String },  // 表示ValidationError变体携带一个名为field的String字段
}

/// 为UserError实现ResponseError trait
///
/// 自定义错误响应和状态码
impl error::ResponseError for UserError {
    /// 当发生UserError错误时，如何生成HTTP响应
    ///
    /// # 返回值
    /// * 返回包含错误信息的HTTP响应
    fn error_response(&self) -> HttpResponse<BoxBody> {
        // 构建HTTP响应，设置状态码和响应头
        HttpResponse::build(self.status_code())  // 设置HTTP状态码（由下方status_code方法决定）
            .content_type("application/json")    // 设置响应头Content-Type为application/json
            .body(self.to_string())              // 响应体为错误的字符串描述（即Display实现的内容）
    }

    /// 指定每种UserError错误对应的HTTP状态码
    ///
    /// # 返回值
    /// * 返回对应错误类型的HTTP状态码
    fn status_code(&self) -> http::StatusCode {
        match self {
            // 如果是ValidationError（验证错误），返回400 Bad Request
            UserError::ValidationError { .. } => http::StatusCode::BAD_REQUEST,
        }
    }
}

/// 内部数据库错误
///
/// 表示数据库操作中发生的错误
/// 用于在内部处理后转换为用户可见错误
#[derive(Debug, Error)]  // 自动派生Debug和Error trait
pub struct InternalDbError;

/// 为InternalDbError实现Display trait
///
/// 提供错误的字符串表示
impl std::fmt::Display for InternalDbError {
    /// 格式化错误消息
    ///
    /// # 参数
    /// * `f` - 格式化器
    ///
    /// # 返回值
    /// * 格式化结果
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "数据库内部错误")  // 写入固定的错误消息
    }
}

/// 用户可见错误
///
/// 表示可以直接展示给用户的错误
/// 隐藏内部实现细节，提供友好的错误消息
#[derive(Debug, Display, Error)]  // 自动派生Debug、Display和Error trait
pub enum UserFacingError {
    #[display(fmt = "发生了内部错误(用户可见错误)")]  // 定义Display输出格式
    InternalError,  // 内部错误变体
}

/// 为UserFacingError实现ResponseError trait
///
/// 自定义错误响应和状态码
impl ResponseError for UserFacingError {
    /// 当发生UserFacingError错误时，如何生成HTTP响应
    ///
    /// # 返回值
    /// * 返回包含用户友好错误信息的HTTP响应
    fn error_response(&self) -> HttpResponse<BoxBody> {
        // 构建HTTP响应
        // 使用InternalServerError()快捷方法创建500状态码响应
        HttpResponse::InternalServerError()
            .content_type("application/json")  // 设置Content-Type为application/json
            .body(self.to_string())            // 响应体为错误的字符串描述
    }

    /// 指定UserFacingError错误对应的HTTP状态码
    ///
    /// # 返回值
    /// * 返回HTTP 500 Internal Server Error状态码
    fn status_code(&self) -> http::StatusCode {
        // 返回HTTP状态码500
        http::StatusCode::INTERNAL_SERVER_ERROR
    }
}
