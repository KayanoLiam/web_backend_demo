use std::sync::Mutex;

use actix_web::middleware::Logger;
use actix_web::{Either, ResponseError, http};
use actix_web::{
    HttpResponse, HttpServer, Responder, Result as ActixResult, body::BoxBody, error, guard, web,
};
use derive_more::{Display, Error};
use futures::StreamExt;
use futures::stream::{self, Stream};
use openssl::ssl::{SslAcceptor, SslFiletype};
// use openssl::string;
use log::info;
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;
use serde_json;
use std::time::Duration;
use tokio::time::interval;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let counter_data = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0),
    });

    //加载ssl证书
    let mut builder = SslAcceptor::mozilla_intermediate(openssl::ssl::SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .expect("Failed to set private key");

    builder
        .set_certificate_chain_file("cert.pem")
        .expect("Failed to set certificate chain");

    unsafe {
        std::env::set_var("RUST_LOG", "info");
        std::env::set_var("RUST_BACKTRACE", "1");
    }
    env_logger::init();

    //创建新的http服务器
    HttpServer::new(move || {
        let logger = Logger::default();

        actix_web::App::new()
            .wrap(logger)
            .app_data(web::Data::new(AppState {
                app_name: "Kayano".to_string(),
            }))
            .app_data(counter_data.clone())
            .configure(config)
            .configure(config_error)
            // .app_data(web::Data::new(AppStateWithCounter {
            //     counter: Mutex::new(0),
            // }))
            // .service(
            //     //这种方式是将路由分组，使用 scope 来分组,这个分组的路由是 /app/index
            //     // web::scope("/app").route("/index", web::get().to(index)),
            //     web::scope("/app")
            //         // .service(web::resource("/index2").to(index2))
            //         // .service(web::resource("/index").to(index))
            //         // .service(web::resource("/index3").to(index3)),
            //         .route("/index2", web::get().to(index2))
            //         .route("/index", web::get().to(index))
            //         .route("/index3", web::get().to(index3))
            //         .guard(guard::Host("users.rust-lang.org"))
            //         ),
            // )
            // .service(
            //     web::scope("/app2")
            //         .guard(guard::Host("www.rust-lang.org"))
            //         .route(
            //             "",
            //             web::to(|| async { HttpResponse::Ok().body("www site") }),
            //         ), // 注意路径是 ""，因为 scope 已经是 "/"
            // )
            .service(first_hello)
            .configure(config2)
            .service(echo)
            .service(path_test)
            .service(path_test_by_struct)
            .service(query_test)
            .service(login)
            .service(my_struct_test)
            .service(stream_handler)
            .service(process_data)
            .service(index_by_my_error)
            .service(process_form)
            // .service(json_test)
            //这种方式是直接在这里定义路由
            .route("/hey", actix_web::web::get().to(manual_hello))
            .service(
                web::resource("/config")
                    .app_data(json_config(4096)) //设置json的最大长度
                    .route(web::post().to(json_test)), //设置路由
            )
    })
    .keep_alive(Duration::from_secs(75)) //设置保持连接的时间
    .workers(10) //设置工作线程数
    .max_connections(100) //设置最大连接数
    .max_connection_rate(10) //设置最大连接速率
    .shutdown_timeout(10) //设置关闭连接的超时时间
    .backlog(100) //设置请求队列的长度
    .shutdown_timeout(60) //设置关闭连接的超时时间
    .bind_openssl("127.0.0.1:8087", builder)?
    .run()
    .await
}

//--------------------------------------接下来都是路由--------------------------------------
#[actix_web::get("/")]
//因为要返回一个响应，所以返回值不能是 ()，而是 impl actix_web::Responder
async fn first_hello() -> impl actix_web::Responder {
    HttpResponse::Ok().body("hello actix-web")
}

#[actix_web::post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body("test ".to_string() + &req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("hello there")
}

//这个函数关联了一个 AppState 结构体
async fn index2(data: web::Data<AppState>) -> String {
    //从 app_state 中获取 app_name
    let app_name = &data.app_name;
    format!("Hello from {}!", app_name)
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body("hello from index")
}

//这个函数关联了一个 AppStateWithCounter 结构体
async fn index3(data: web::Data<AppStateWithCounter>) -> String {
    let mut counter = data.counter.lock().unwrap();
    *counter += 1;
    format!("Hello from index3! Counter: {}", counter)
}

//这个路由使用curl -k https://127.0.0.1:8087/path/32/yourname 测试，注意是https
#[actix_web::get("/path/{user_id}/{name}")]
async fn path_test(path: web::Path<(u32, String)>) -> Result<String, actix_web::Error> {
    //获取路径参数
    let (user_id, name) = path.into_inner(); //这个into_inner()方法是将路径参数转换为元组
    Ok(format!(
        "Hello from Path_test! User ID: {}, Name: {}",
        user_id, name
    ))
}

//这个函数关联了一个 UserInfo 结构体
//这个函数使用curl -k https://127.0.0.1:8087/path2/32/yourname
#[actix_web::get("/path2/{user_id}/{name}")]
async fn path_test_by_struct(path: web::Path<UserInfo>) -> Result<String, actix_web::Error> {
    //获取路径参数
    let user_info = path.into_inner(); //这个into_inner()方法是将路径参数转换为结构体
    Ok(format!(
        "Hello from Path_test! User ID: {}, Name: {}",
        user_info.user_id, user_info.name
    ))
}
//这个函数关联了一个 SearchQuery 结构体
//这个函数使用curl -k https://127.0.0.1:8087/query?q=your_query&lang=your_lang
#[actix_web::get("/query")]
// 该异步处理函数用于处理 GET /query 路由，接收查询参数并返回响应字符串
async fn query_test(query: web::Query<SearchQuery>) -> String {
    // 检查查询参数中是否包含 lang 字段
    if let Some(lang) = &query.lang {
        // 如果包含 lang 字段，返回带有 lang 的响应字符串
        format!("Hello from query_test! Query: {}, lang: {}", query.q, lang)
    } else {
        // 如果不包含 lang 字段，仅返回 q 字段
        format!("Hello from query_test! Query: {}", query.q)
    }
}

//这个函数关联了一个 UserIput 结构体
//这个函数使用curl -k -X POST -H "Content-Type: application/json" -d '{"username":"your_username","email":"your_email"}' https://127.0.0.1:8087/json
// #[actix_web::post("/json_test")] 这个注释掉的路由是因为在上面的配置函数中已经配置了
async fn json_test(user: web::Json<UserIput>) -> String {
    //获取json参数
    let user = user.into_inner(); //这个into_inner()方法是将json参数转换为结构体
    format!(
        "Hello from json_test! Username: {}, Email: {}",
        user.username, user.email
    )
}

#[actix_web::post("/login")]
//这个函数使用curl -k -X POST -H "Content-Type: application/x-www-form-urlencoded" -d 'username=your_username&password=your_password' https://127.0.0.1:8087/login
async fn login(form: web::Form<LoginInfo>) -> String {
    //获取表单参数
    let login_info = form.into_inner(); //这个into_inner()方法是将表单参数转换为结构体
    format!(
        "Hello from login! Username: {}, Password: {}",
        login_info.username, login_info.password
    )
}

#[actix_web::get("/my_struct")]
async fn my_struct_test() -> impl Responder {
    MyStruct {
        name: "Kayano".to_string(),
        age: 18,
    }
}

#[actix_web::get("/sse")]
async fn stream_handler() -> HttpResponse {
    let stream = create_sse_stream(); //这里的这个create_sse_stream是一个函数，返回一个流
    HttpResponse::Ok()
        .content_type("text/event-stream")
        .streaming(stream)
}

/// 处理 GET /process 路由，根据随机结果返回不同响应。
/// - 70% 概率返回 JSON 格式的 MyStruct 数据（模拟成功）
/// - 30% 概率返回 500 错误响应（模拟失败）
/// 可用 curl -k https://127.0.0.1:8087/process 测试
#[actix_web::get("/process")]
async fn process_data() -> ProcessResult {
    let success = rand::thread_rng().gen_bool(0.7);
    if success {
        println!("success");
        Either::Left(MyStruct {
            name: "Kayano".to_string(),
            age: 18,
        })
    } else {
        Either::Right(HttpResponse::InternalServerError().body("error"))
    }
}

#[actix_web::get("/first_error")]
async fn index_by_my_error() -> Result<&'static str, MyError> {
    info!("使用info日志记录");
    Err(MyError {
        name: "测试错误"
    })
}

#[actix_web::get("/internal_error")]
async fn index_by_my_new_error_internal() -> Result<&'static str, MyNewError> {
    Err(MyNewError::InternalError)
}

#[actix_web::get("/timeout")]
async fn index_by_my_new_error_timeout() -> Result<&'static str, MyNewError> {
    Err(MyNewError::Timeout)
}
#[actix_web::get("/bad_client_data")]
async fn index_by_my_new_error_bad_client_data() -> Result<&'static str, MyNewError> {
    Err(MyNewError::BadClientData)
}

#[actix_web::get("/simple_error")] // 定义 GET 路由 /simple_error，访问该路径会调用此函数
async fn index_by_simple_error() -> ActixResult<String> {
    // 创建一个 Result 类型的变量 result，模拟一个错误（Err），错误类型为 MySimpleError
    let result: Result<String, MySimpleError> = Err(MySimpleError {
        name: "测试错误"
    });
    // 将 result 的错误类型 MySimpleError 映射为 actix_web 的错误类型
    result.map_err(|err| {
        // 如果发生错误，将错误信息转换为 HTTP 400 Bad Request 响应
        error::ErrorBadRequest(err.name)
    })
}

#[actix_web::get("/form_test")]
async fn process_form() -> Result<&'static str, UserError> {
    let user_input = "invalid_email"; // 假设这是用户输入
    if !user_input.contains('@') {
        // 简单的验证
        // 返回一个用户可见的错误
        return Err(UserError::ValidationError {
            field: "email".to_string(),
        });
    }
    Ok("处理成功")
}

#[actix_web::get("/user_facing_error")]
async fn index_by_user_facing_error() -> ActixResult<&'static str, UserFacingError> {
    do_thing_that_may_fail().map_err(|_| {
        // 如果发生错误，返回一个用户可见的错误
        UserFacingError::InternalError
    })?;
    Ok("处理成功")
}
//--------------------------------------接下来都是结构体--------------------------------------
struct AppState {
    app_name: String,
}

struct AppStateWithCounter {
    counter: Mutex<i32>,
}

//这个结构体是用来解析路径参数的,凡是要解析路径参数的结构体都要加上这个宏
#[derive(Deserialize)]
struct UserInfo {
    user_id: u32,
    name: String,
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    lang: Option<String>,
}

#[derive(Deserialize)]
struct UserIput {
    username: String,
    email: String,
}

#[derive(Deserialize)]
struct LoginInfo {
    username: String,
    password: String,
}

//Serialize 是一个用于将数据结构序列化为 JSON 的 trait，而Deserialize 是一个用于将 JSON 反序列化为数据结构的 trait
//它俩是相反的，Serialize 是将数据结构转换为 JSON 字符串，而 Deserialize 是将 JSON 字符串转换为数据结构
#[derive(Serialize)]
struct MyStruct {
    name: String,
    age: u32,
}

#[derive(Debug, Display, Error)]
struct MyError {
    name: &'static str,
}

#[derive(Debug, Display, Error)]
enum MyNewError {
    #[display(fmt = "内部错误")]
    InternalError,
    #[display(fmt = "请求超时")]
    Timeout,
    #[display(fmt = "请求错误")]
    BadClientData,
}

#[derive(Debug)]
struct MySimpleError {
    name: &'static str,
}

#[derive(Debug, Display, Error)]
enum UserError {
    #[display(fmt = "验证错误: {field}")]
    // 这里的花括号用于定义结构体变体（struct variant），
    // 表示 ValidationError 变体携带一个名为 field 的 String 字段
    ValidationError { field: String },
}

#[derive(Debug, Error)]
struct InternalDbError;

#[derive(Debug, Display, Error)]
enum UserFacingError {
    #[display(fmt = "发生了内部错误(用户可见错误)")]
    InternalError,
}
//---------------------------------------接下来都是配置--------------------------------------
//这是app的配置函数
fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/app")
            .route("/index", web::get().to(index))
            .route("/index2", web::get().to(index2))
            .route(
                "",
                web::get()
                    .guard(guard::Host("users.rust-lang.org"))
                    .to(|| async { HttpResponse::Ok().body("users site") }),
            ),
    );
}

//这是app2的配置函数,与上面的配置函数是一样的
fn config2(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/app2")
            .route("/index", web::get().to(index))
            .route("/index3", web::get().to(index3))
            .route(
                //测试这个守卫路由使用： curl -H "Host: www.rust-lang.org" http://127.0.0.1:8087/app2
                "",
                web::get()
                    .guard(guard::Host("www.rust-lang.org"))
                    .to(|| async { HttpResponse::Ok().body("www site") }),
            ),
    );
}

//这个函数是用来设置json的配置的
fn json_config(limit: usize) -> web::JsonConfig {
    //设置json的最大长度
    web::JsonConfig::default()
        .limit(limit)
        .error_handler(|err, _| {
            //处理json解析错误
            println!("json error: {}", err);
            error::InternalError::from_response(err, HttpResponse::Conflict().body("json error"))
                .into()
        })
}

fn config_error(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/error")
            .service(index_by_my_new_error_internal)
            .service(index_by_my_new_error_timeout)
            .service(index_by_my_new_error_bad_client_data)
            .service(index_by_simple_error)
            .service(index_by_user_facing_error)
    );
}
//----------------------------------------接下来都是为结构体实现trait--------------------------------------
impl std::fmt::Display for InternalDbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "数据库内部错误")
    }
}

impl ResponseError for UserFacingError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        // 构建 HTTP 响应，设置 Content-Type 为 application/json，并将错误信息作为响应体
        HttpResponse::InternalServerError()
            .content_type("application/json")
            .body(self.to_string())
    }
    fn status_code(&self) -> http::StatusCode {
        // 返回 HTTP 状态码 500
        http::StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl Responder for MyStruct {
    // 指定响应体的类型为 BoxBody，这是 actix-web 推荐的响应体类型
    type Body = BoxBody;

    // 实现 respond_to 方法，将 MyStruct 转换为 HTTP 响应
    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        // 将结构体序列化为 JSON 字符串
        let body = serde_json::to_string(&self).unwrap();
        // 构建 HTTP 响应，设置 Content-Type 为 application/json，并将序列化后的 JSON 作为响应体
        HttpResponse::Ok()
            .content_type("application/json")
            .body(body)
    }
}

impl error::ResponseError for MyError {}

impl error::ResponseError for MyNewError {
    // 定义当错误发生时如何生成 HTTP 响应
    fn error_response(&self) -> HttpResponse<BoxBody> {
        // 使用 self.status_code() 获取对应的 HTTP 状态码，构建响应
        HttpResponse::build(self.status_code())
            // 设置响应头 Content-Type 为 application/json
            .content_type("application/json")
            // 响应体内容为错误的字符串描述（即枚举的 Display 实现）
            .body(self.to_string())
    }
    // 定义每种错误对应的 HTTP 状态码
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            // InternalError 映射为 500 Internal Server Error
            MyNewError::InternalError => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            // Timeout 映射为 408 Request Timeout
            MyNewError::Timeout => actix_web::http::StatusCode::REQUEST_TIMEOUT,
            // BadClientData 映射为 400 Bad Request
            MyNewError::BadClientData => actix_web::http::StatusCode::BAD_REQUEST,
        }
    }
}

impl error::ResponseError for UserError {
    /// 当发生 UserError 错误时，如何生成 HTTP 响应
    fn error_response(&self) -> HttpResponse<BoxBody> {
        // 构建 HTTP 响应，设置状态码和响应头
        HttpResponse::build(self.status_code()) // 设置 HTTP 状态码（由下方 status_code 方法决定）
            .content_type("application/json")   // 设置响应头 Content-Type 为 application/json
            .body(self.to_string())             // 响应体为错误的字符串描述（即 Display 实现的内容）
    }

    /// 指定每种 UserError 错误对应的 HTTP 状态码
    fn status_code(&self) -> http::StatusCode {
        match self {
            // 如果是 ValidationError（验证错误），返回 400 Bad Request
            UserError::ValidationError { .. } => http::StatusCode::BAD_REQUEST,
        }
    }
}
//-----------------------------------------接下来是非路由函数--------------------------------------
fn create_sse_stream() -> impl Stream<Item = Result<web::Bytes, std::io::Error>> {
    // 定义一个计数器，用于生成递增的数据
    let mut counter: usize = 0;
    // 创建一个定时器，每隔1秒触发一次
    let mut interval = interval(Duration::from_secs(1));
    // 使用 poll_fn 创建一个自定义流
    stream::poll_fn(move |cx| match interval.poll_tick(cx) {
        // 如果定时器触发（每秒一次）
        std::task::Poll::Ready(_) => {
            counter += 1; // 计数器加1
            // 构造 SSE 格式的数据字符串
            let msg = format!("data: {}\n\n", counter);
            // 返回数据（注意这里原本应返回 String，但实际返回了 Bytes，类型不符，建议修正）
            std::task::Poll::Ready(Some(Ok(web::Bytes::from(msg))))
        }
        // 如果定时器还没到时间，则挂起等待
        std::task::Poll::Pending => std::task::Poll::Pending,
    })
    // 限制流最多产生10条消息
    .take(10)
}

type ProcessResult = Either<MyStruct, HttpResponse>;

fn do_thing_that_may_fail() -> Result<(), InternalDbError> {
    // 这里是一些可能会失败的操作
    // 如果失败，返回一个 InternalDbError 错误
    Err(InternalDbError)
}
