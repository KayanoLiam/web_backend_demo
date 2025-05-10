use std::sync::Mutex;

use actix_web::{HttpResponse, HttpServer, Responder, guard, web};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let counter_data = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0),
    });

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
            //这种方式是直接在这里定义路由
            .route("/hey", actix_web::web::get().to(manual_hello))
    })
    .bind("127.0.0.1:8087")?
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
//--------------------------------------接下来都是结构体--------------------------------------
struct AppState {
    app_name: String,
}

struct AppStateWithCounter {
    counter: Mutex<i32>,
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
