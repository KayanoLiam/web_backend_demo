// 外部库导入
use actix_web::{error, guard, web, HttpResponse};  // 用于Web应用配置和HTTP响应

// 内部模块导入
// 导入各种路由处理函数
use crate::handlers::{
    // 基本页面处理函数
    index, index2, index3,
    // 错误演示处理函数
    index_by_my_new_error_internal,
    index_by_my_new_error_timeout,
    index_by_my_new_error_bad_client_data,
    index_by_simple_error,
    index_by_user_facing_error
};

/// 应用主路由配置函数
///
/// 配置/app路径下的路由
/// 包括基本页面和带有主机守卫的路由
///
/// # 参数
/// * `cfg` - 服务配置引用，用于注册路由
pub fn config(cfg: &mut web::ServiceConfig) {
    // 注册/app路径下的所有路由
    cfg.service(
        // 创建一个作用域为"/app"的路由组
        web::scope("/app")
            // 注册GET /app/index路由，使用index处理函数
            .route("/index", web::get().to(index))

            // 注册GET /app/index2路由，使用index2处理函数
            .route("/index2", web::get().to(index2))

            // 注册带有主机守卫的路由
            .route(
                // 空路径表示/app本身
                "",
                web::get()
                    // 只有当Host头为"users.rust-lang.org"时才匹配
                    .guard(guard::Host("users.rust-lang.org"))
                    // 使用内联异步闭包处理请求
                    .to(|| async { HttpResponse::Ok().body("users site") }),
            ),
    );
}

/// 第二应用路由配置函数
///
/// 配置/app2路径下的路由
/// 包括基本页面和带有主机守卫的路由
///
/// # 参数
/// * `cfg` - 服务配置引用，用于注册路由
pub fn config2(cfg: &mut web::ServiceConfig) {
    // 注册/app2路径下的所有路由
    cfg.service(
        // 创建一个作用域为"/app2"的路由组
        web::scope("/app2")
            // 注册GET /app2/index路由，使用index处理函数
            .route("/index", web::get().to(index))

            // 注册GET /app2/index3路由，使用index3处理函数
            .route("/index3", web::get().to(index3))

            // 注册带有主机守卫的路由
            .route(
                // 空路径表示/app2本身
                // 测试这个守卫路由使用： curl -H "Host: www.rust-lang.org" http://127.0.0.1:8087/app2
                "",
                web::get()
                    // 只有当Host头为"www.rust-lang.org"时才匹配
                    .guard(guard::Host("www.rust-lang.org"))
                    // 使用内联异步闭包处理请求
                    .to(|| async { HttpResponse::Ok().body("www site") }),
            ),
    );
}

/// 错误处理路由配置函数
///
/// 配置/error路径下的路由
/// 包含各种错误演示处理函数
///
/// # 参数
/// * `cfg` - 服务配置引用，用于注册路由
pub fn config_error(cfg: &mut web::ServiceConfig) {
    // 注册/error路径下的所有路由
    cfg.service(
        // 创建一个作用域为"/error"的路由组
        web::scope("/error")
            // 注册各种错误演示路由

            // 内部错误演示
            .service(index_by_my_new_error_internal)

            // 超时错误演示
            .service(index_by_my_new_error_timeout)

            // 客户端数据错误演示
            .service(index_by_my_new_error_bad_client_data)

            // 简单错误演示
            .service(index_by_simple_error)

            // 用户可见错误演示
            .service(index_by_user_facing_error)
    );
}

/// JSON配置函数
///
/// 创建自定义JSON配置，设置最大请求体大小和错误处理
///
/// # 参数
/// * `limit` - JSON请求体的最大字节数
///
/// # 返回值
/// * 返回配置好的JsonConfig实例
pub fn json_config(limit: usize) -> web::JsonConfig {
    // 从默认配置开始
    web::JsonConfig::default()
        // 设置JSON请求体的最大长度
        .limit(limit)
        // 自定义错误处理器
        .error_handler(|err, _| {
            // 处理JSON解析错误
            // 打印错误信息到控制台
            println!("JSON error: {}", err);

            // 创建一个内部错误，返回409 Conflict状态码
            // 将原始错误包装在自定义响应中
            error::InternalError::from_response(
                err,
                HttpResponse::Conflict().body("JSON error")
            )
            .into()  // 转换为actix_web::Error类型
        })
}
