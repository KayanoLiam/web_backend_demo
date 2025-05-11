use std::sync::Mutex;

use actix_web::{HttpResponse, HttpServer, Responder, error, guard, web};
use openssl::ssl::{SslAcceptor, SslFiletype};
use serde::Deserialize;
use std::time::Duration;
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

    //创建新的http服务器
    HttpServer::new(move || {
        actix_web::App::new()
            .app_data(web::Data::new(AppState {
                app_name: "Kayano".to_string(),
            }))
            .app_data(counter_data.clone())
            .configure(config)
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
            // .service(json_test)
            //这种方式是直接在这里定义路由
            .route("/hey", actix_web::web::get().to(manual_hello))
            .service(
                web::resource("/config")
                .app_data(json_config(4096)) //设置json的最大长度
                .route(web::post().to(json_test)) //设置路由
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
