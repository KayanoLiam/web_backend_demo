// 导入自定义模块
mod config;    // 包含应用程序的配置函数
mod errors;    // 包含自定义错误类型和实现
mod handlers;  // 包含所有HTTP请求处理函数
mod models;    // 包含数据模型和结构体
mod utils;     // 包含工具函数

// 标准库导入
use std::sync::Mutex;  // 用于线程安全的共享状态
use std::time::Duration;  // 用于设置时间相关的配置

// 外部库导入
use actix_web::middleware::Logger;  // 用于请求日志记录
use actix_web::{guard, web, HttpServer};   // Web服务器和Web相关工具
use openssl::ssl::{SslAcceptor, SslFiletype};
// use rand::seq::index;  // SSL/TLS支持

// 从自定义模块导入特定组件
// 导入配置函数
use crate::config::{config, config_error, config2, json_config};
// 导入所有HTTP请求处理函数
use crate::handlers::{
    echo, first_hello, index_by_my_error, login, manual_hello, my_struct_test, path_test,
    path_test_by_struct, process_data, process_form, query_test, stream_handler,index_resource,
    get_user, updata_user
};
// 导入应用状态结构体
use crate::models::{AppState, AppStateWithCounter};

/// 应用程序入口点
///
/// 使用actix_web宏将异步函数标记为应用程序入口点
/// 配置并启动HTTPS服务器，设置路由和中间件
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 创建一个计数器状态，用于在请求之间共享
    let counter_data = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0), // 初始化为0的线程安全计数器
    });

    // 加载SSL证书，配置HTTPS支持
    // 使用Mozilla推荐的中间安全级别配置
    let mut builder = SslAcceptor::mozilla_intermediate(openssl::ssl::SslMethod::tls()).unwrap();

    // 设置私钥文件
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .expect("Failed to set private key");

    // 设置证书链文件
    builder
        .set_certificate_chain_file("cert.pem")
        .expect("Failed to set certificate chain");

    // 配置日志环境变量
    // 使用unsafe块是因为环境变量修改是全局性的
    unsafe {
        std::env::set_var("RUST_LOG", "info");       // 设置日志级别为info
        std::env::set_var("RUST_BACKTRACE", "1");    // 启用错误回溯
    }

    // 初始化日志系统
    env_logger::init();

    // 创建新的HTTP服务器
    // move关键字将counter_data所有权移入闭包
    HttpServer::new(move || {
        // 创建默认日志记录器
        let logger = Logger::default();

        // 创建新的应用实例，配置中间件和路由
        actix_web::App::new()
            // 添加日志中间件
            .wrap(logger)
            // 添加应用状态数据
            .app_data(web::Data::new(AppState {
                app_name: "Kayano".to_string(),  // 设置应用名称
            }))
            // 添加计数器状态数据
            .app_data(counter_data.clone())

            // 配置路由组
            .configure(config)         // 配置/app路径下的路由
            .configure(config_error)   // 配置/error路径下的路由

            // 注册各个路由处理函数
            .service(first_hello)          // 处理根路径"/"
            .configure(config2)            // 配置/app2路径下的路由
            .service(echo)                 // 处理POST /echo
            .service(path_test)            // 处理GET /path/{user_id}/{name}
            .service(path_test_by_struct)  // 处理GET /path2/{user_id}/{name}
            .service(query_test)           // 处理GET /query
            .service(login)                // 处理POST /login
            .service(my_struct_test)       // 处理GET /my_struct
            .service(stream_handler)       // 处理GET /sse
            .service(process_data)         // 处理GET /process
            .service(index_by_my_error)    // 处理GET /first_error
            .service(process_form)         // 处理GET /form_test
            // 注册一个简单的资源路由，路径为"/perix"
            // 当访问 /perix 时，所有HTTP方法的请求都会被转发到index_resource处理函数
            .service(web::resource("/perix").to(index_resource))
            
            // 注册一个带路径参数的复杂资源路由
            .service(
                // 定义资源路径为"user/{name}"，其中{name}是动态路径参数
                // 例如：user/alice、user/bob等都会匹配这个路由
                web::resource("user/{name}")
                // 为该路由指定名称"user_detail"，可用于反向URL生成
                // 例如：req.url_for("user_detail", &["alice"]) 会生成 /user/alice
                .name("user_detail")
                // 添加请求守卫(guard)，只有当请求头中的Content-Type为"application/json"时才会匹配该路由
                // 如果请求头不符合条件，路由匹配会失败，请求会继续尝试匹配其他路由
                // 这对于确保只处理特定格式的请求非常有用，例如只接受JSON格式的数据
                .guard(guard::Header("content-type", "application/json"))
                // 配置GET请求的处理函数
                // 当收到 GET /user/{name} 请求时，调用get_user函数处理
                .route(web::get().to(get_user))
                // 配置PUT请求的处理函数
                // 当收到 PUT /user/{name} 请求时，调用updata_user函数处理
                // 注：这里可能是拼写错误，应为update_user而非updata_user
                .route(web::put().to(updata_user))
            )
            // 手动注册路由，不使用宏
            .route("/hey", web::get().to(manual_hello))
            
            // 配置带有JSON配置的路由
            .service(
                web::resource("/config")
                    .app_data(json_config(4096))  // 设置JSON请求体最大长度为4096字节
                    .route(web::post().to(handlers::json_test)),  // 设置POST处理函数
            )
    })
    // 服务器全局配置
    .keep_alive(Duration::from_secs(75))    // 设置保持连接的时间为75秒
    .workers(10)                            // 设置工作线程数为10
    .max_connections(100)                   // 设置最大连接数为100
    .max_connection_rate(10)                // 设置最大连接速率为每秒10个
    .shutdown_timeout(10)                   // 设置关闭连接的超时时间为10秒
    .backlog(100)                           // 设置请求队列的长度为100
    .shutdown_timeout(60)                   // 设置关闭服务器的超时时间为60秒
    .bind_openssl("127.0.0.1:8087", builder)? // 绑定到127.0.0.1:8087，使用SSL
    .run()                                  // 运行服务器
    .await                                  // 等待服务器运行完成
}
