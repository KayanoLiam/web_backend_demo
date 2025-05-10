use actix_web::{HttpResponse, HttpServer,Responder};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| actix_web::App::new().service(first_hello))
        .bind("127.0.0.1:8087")?
        .run()
        .await
}

//--------------------------------------
#[actix_web::get("/")]
//因为要返回一个响应，所以返回值不能是 ()，而是 impl actix_web::Responder
async fn first_hello() -> impl actix_web::Responder {
    HttpResponse::Ok().body("hello actix-web")
}

#[actix_web::get("/echo")]
async fn echo(req_body:String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}