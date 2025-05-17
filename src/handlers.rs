// 外部库导入
use actix_web::{Either,HttpRequest, HttpResponse, Responder, Result as ActixResult, error, web};  // Web框架核心组件
use log::info;  // 日志记录
use rand::Rng;  // 随机数生成

// 内部模块导入
// 导入数据模型
use crate::models::{
    AppState, AppStateWithCounter,  // 应用状态结构体
    LoginInfo, MyStruct,            // 登录信息和响应结构体
    SearchQuery, UserInfo, UserIput // 查询参数和用户信息结构体
};
// 导入错误类型
use crate::errors::{
    MyError, MyNewError, MySimpleError,  // 基本错误类型
    UserError, UserFacingError           // 用户相关错误类型
};
// 导入工具函数
use crate::utils::{create_sse_stream, do_thing_that_may_fail};

/// 处理结果类型别名
///
/// 表示处理函数可能返回两种结果之一：
/// 1. 成功时返回MyStruct结构体
/// 2. 失败时返回HTTP错误响应
pub type ProcessResult = Either<MyStruct, HttpResponse>;

/// 路由处理函数部分
/// 包含各种HTTP请求处理函数

/// 根路径处理函数
///
/// 处理GET /请求，返回简单的欢迎消息
/// 使用宏注册路由
///
/// # 返回值
/// * 返回包含文本消息的HTTP 200 OK响应
#[actix_web::get("/")]
pub async fn first_hello() -> impl actix_web::Responder {
    HttpResponse::Ok().body("hello actix-web")
}

/// Echo处理函数
///
/// 处理POST /echo请求，将请求体内容作为响应返回
/// 使用宏注册路由
///
/// # 参数
/// * `req_body` - 请求体内容，自动提取为String
///
/// # 返回值
/// * 返回包含请求体内容的HTTP 200 OK响应
#[actix_web::post("/echo")]
pub async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body("test ".to_string() + &req_body)
}

/// 手动注册的Hello处理函数
///
/// 处理GET /hey请求，返回简单的欢迎消息
/// 通过手动注册路由，而不是使用宏
///
/// # 返回值
/// * 返回包含文本消息的HTTP 200 OK响应
pub async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("hello there")
}

/// 带应用状态的处理函数
///
/// 处理GET /app/index2请求，从应用状态中获取应用名称
/// 演示如何访问共享应用状态
///
/// # 参数
/// * `data` - 应用状态，通过依赖注入获取
///
/// # 返回值
/// * 返回包含应用名称的字符串
pub async fn index2(data: web::Data<AppState>) -> String {
    // 从app_state中获取app_name
    let app_name = &data.app_name;
    format!("Hello from {}!", app_name)
}

/// 基本索引处理函数
///
/// 处理GET /app/index请求，返回简单的欢迎消息
///
/// # 返回值
/// * 返回包含文本消息的HTTP 200 OK响应
pub async fn index() -> impl Responder {
    HttpResponse::Ok().body("hello from index")
}

/// 带计数器的处理函数
///
/// 处理GET /app2/index3请求，递增计数器并返回当前值
/// 演示如何修改共享状态
///
/// # 参数
/// * `data` - 带计数器的应用状态，通过依赖注入获取
///
/// # 返回值
/// * 返回包含计数器值的字符串
pub async fn index3(data: web::Data<AppStateWithCounter>) -> String {
    // 获取计数器的互斥锁
    let mut counter = data.counter.lock().unwrap();
    // 递增计数器
    *counter += 1;
    // 返回包含计数器值的消息
    format!("Hello from index3! Counter: {}", counter)
}

/// 路径参数处理函数（使用元组）
///
/// 处理GET /path/{user_id}/{name}请求，从URL路径中提取参数
/// 演示如何使用元组提取路径参数
///
/// # 参数
/// * `path` - 路径参数，自动提取为(u32, String)元组
///
/// # 返回值
/// * 成功时返回包含用户ID和名称的字符串
/// * 失败时返回actix_web错误
#[actix_web::get("/path/{user_id}/{name}")]
pub async fn path_test(path: web::Path<(u32, String)>) -> Result<String, actix_web::Error> {
    // 获取路径参数
    // into_inner()方法将路径参数转换为元组
    let (user_id, name) = path.into_inner();

    // 返回格式化的响应
    Ok(format!(
        "Hello from Path_test! User ID: {}, Name: {}",
        user_id, name
    ))
}

/// 路径参数处理函数（使用结构体）
///
/// 处理GET /path2/{user_id}/{name}请求，从URL路径中提取参数
/// 演示如何使用结构体提取路径参数
///
/// # 参数
/// * `path` - 路径参数，自动提取为UserInfo结构体
///
/// # 返回值
/// * 成功时返回包含用户ID和名称的字符串
/// * 失败时返回actix_web错误
#[actix_web::get("/path2/{user_id}/{name}")]
pub async fn path_test_by_struct(path: web::Path<UserInfo>) -> Result<String, actix_web::Error> {
    // 获取路径参数
    // into_inner()方法将路径参数转换为结构体
    let user_info = path.into_inner();

    // 返回格式化的响应
    Ok(format!(
        "Hello from Path_test! User ID: {}, Name: {}",
        user_info.user_id, user_info.name
    ))
}

/// 查询参数处理函数
///
/// 处理GET /query?q=xxx&lang=yyy请求，从URL查询字符串中提取参数
/// 演示如何处理必需和可选的查询参数
///
/// # 参数
/// * `query` - 查询参数，自动提取为SearchQuery结构体
///
/// # 返回值
/// * 返回包含查询参数的字符串
#[actix_web::get("/query")]
pub async fn query_test(query: web::Query<SearchQuery>) -> String {
    // 检查查询参数中是否包含lang字段
    if let Some(lang) = &query.lang {
        // 如果包含lang字段，返回带有lang的响应字符串
        format!("Hello from query_test! Query: {}, lang: {}", query.q, lang)
    } else {
        // 如果不包含lang字段，仅返回q字段
        format!("Hello from query_test! Query: {}", query.q)
    }
}

/// JSON请求处理函数
///
/// 处理POST /config请求，从请求体中提取JSON数据
/// 演示如何处理JSON请求体
///
/// # 参数
/// * `user` - JSON请求体，自动提取为UserIput结构体
///
/// # 返回值
/// * 返回包含用户名和邮箱的字符串
pub async fn json_test(user: web::Json<UserIput>) -> String {
    // 获取JSON参数
    // into_inner()方法将JSON参数转换为结构体
    let user = user.into_inner();

    // 返回格式化的响应
    format!(
        "Hello from json_test! Username: {}, Email: {}",
        user.username, user.email
    )
}

/// 表单处理函数
///
/// 处理POST /login请求，从请求体中提取表单数据
/// 演示如何处理表单提交
///
/// # 参数
/// * `form` - 表单数据，自动提取为LoginInfo结构体
///
/// # 返回值
/// * 返回包含用户名和密码的字符串
#[actix_web::post("/login")]
pub async fn login(form: web::Form<LoginInfo>) -> String {
    // 获取表单参数
    // into_inner()方法将表单参数转换为结构体
    let login_info = form.into_inner();

    // 返回格式化的响应
    format!(
        "Hello from login! Username: {}, Password: {}",
        login_info.username, login_info.password
    )
}

/// 结构体响应处理函数
///
/// 处理GET /my_struct请求，返回JSON格式的结构体
/// 演示如何返回自定义结构体作为JSON响应
///
/// # 返回值
/// * 返回MyStruct结构体，自动序列化为JSON
#[actix_web::get("/my_struct")]
pub async fn my_struct_test() -> impl Responder {
    // 创建并返回MyStruct实例
    // 会自动序列化为JSON响应
    MyStruct {
        name: "Kayano".to_string(),  // 设置名称
        age: 18,                      // 设置年龄
    }
}

/// 服务器发送事件(SSE)处理函数
///
/// 处理GET /sse请求，返回实时更新的事件流
/// 演示如何实现服务器推送功能
///
/// # 返回值
/// * 返回包含事件流的HTTP响应
#[actix_web::get("/sse")]
pub async fn stream_handler() -> HttpResponse {
    // 创建SSE流
    // create_sse_stream函数返回一个每秒发送一次数据的流
    let stream = create_sse_stream();

    // 返回流式响应
    HttpResponse::Ok()
        .content_type("text/event-stream")  // 设置SSE内容类型
        .streaming(stream)                  // 使用流作为响应体
}

/// 随机处理结果函数
///
/// 处理GET /process请求，随机返回成功或失败
/// 演示如何使用Either类型返回不同类型的响应
///
/// # 返回值
/// * 70%概率返回MyStruct结构体（成功）
/// * 30%概率返回HTTP 500错误（失败）
#[actix_web::get("/process")]
pub async fn process_data() -> ProcessResult {
    // 生成随机布尔值，70%概率为true
    let success = rand::thread_rng().gen_bool(0.7);

    if success {
        // 成功情况
        println!("success");
        // 返回MyStruct实例
        Either::Left(MyStruct {
            name: "Kayano".to_string(),
            age: 18,
        })
    } else {
        // 失败情况
        // 返回HTTP 500错误响应
        Either::Right(HttpResponse::InternalServerError().body("error"))
    }
}

/// 简单错误演示函数
///
/// 处理GET /first_error请求，总是返回错误
/// 演示如何返回自定义错误
///
/// # 返回值
/// * 总是返回MyError错误
#[actix_web::get("/first_error")]
pub async fn index_by_my_error() -> Result<&'static str, MyError> {
    // 记录日志
    info!("使用info日志记录");

    // 返回错误
    Err(MyError {
        name: "测试错误"  // 设置错误消息
    })
}

/// 内部错误演示函数
///
/// 处理GET /error/internal_error请求，总是返回内部错误
/// 演示如何返回枚举错误的特定变体
///
/// # 返回值
/// * 总是返回MyNewError::InternalError错误
#[actix_web::get("/internal_error")]
pub async fn index_by_my_new_error_internal() -> Result<&'static str, MyNewError> {
    // 返回内部错误
    Err(MyNewError::InternalError)
}

/// 超时错误演示函数
///
/// 处理GET /error/timeout请求，总是返回超时错误
/// 演示如何返回枚举错误的特定变体
///
/// # 返回值
/// * 总是返回MyNewError::Timeout错误
#[actix_web::get("/timeout")]
pub async fn index_by_my_new_error_timeout() -> Result<&'static str, MyNewError> {
    // 返回超时错误
    Err(MyNewError::Timeout)
}

/// 客户端数据错误演示函数
///
/// 处理GET /error/bad_client_data请求，总是返回客户端数据错误
/// 演示如何返回枚举错误的特定变体
///
/// # 返回值
/// * 总是返回MyNewError::BadClientData错误
#[actix_web::get("/bad_client_data")]
pub async fn index_by_my_new_error_bad_client_data() -> Result<&'static str, MyNewError> {
    // 返回客户端数据错误
    Err(MyNewError::BadClientData)
}

/// 简单错误映射演示函数
///
/// 处理GET /error/simple_error请求，演示错误映射
/// 展示如何将自定义错误映射为actix_web错误
///
/// # 返回值
/// * 总是返回映射后的错误
#[actix_web::get("/simple_error")]
pub async fn index_by_simple_error() -> ActixResult<String> {
    // 创建一个Result类型的变量result，模拟一个错误（Err），错误类型为MySimpleError
    let result: Result<String, MySimpleError> = Err(MySimpleError {
        name: "测试错误"
    });

    // 将result的错误类型MySimpleError映射为actix_web的错误类型
    result.map_err(|err| {
        // 如果发生错误，将错误信息转换为HTTP 400 Bad Request响应
        error::ErrorBadRequest(err.name)
    })
}

/// 表单验证错误演示函数
///
/// 处理GET /form_test请求，演示表单验证错误
/// 展示如何进行简单验证并返回适当的错误
///
/// # 返回值
/// * 验证失败时返回UserError::ValidationError错误
/// * 验证成功时返回成功消息
#[actix_web::get("/form_test")]
pub async fn process_form() -> Result<&'static str, UserError> {
    // 假设这是用户输入
    let user_input = "invalid_email";

    // 简单的验证：检查是否包含@符号
    if !user_input.contains('@') {
        // 验证失败，返回一个用户可见的错误
        return Err(UserError::ValidationError {
            field: "email".to_string(),  // 指定验证失败的字段
        });
    }

    // 验证成功
    Ok("处理成功")
}

/// 用户可见错误演示函数
///
/// 处理GET /error/user_facing_error请求，演示错误转换
/// 展示如何将内部错误转换为用户可见错误
///
/// # 返回值
/// * 总是返回UserFacingError::InternalError错误
#[actix_web::get("/user_facing_error")]
pub async fn index_by_user_facing_error() -> Result<&'static str, UserFacingError> {
    // 调用可能失败的函数，并将内部错误转换为用户可见错误
    do_thing_that_may_fail().map_err(|_| {
        // 如果发生错误，返回一个用户可见的错误
        UserFacingError::InternalError
    })?;

    // 如果没有错误（实际上这里永远不会执行，因为do_thing_that_may_fail总是返回错误）
    Ok("处理成功")
}


pub async fn index_resource() -> HttpResponse {
    HttpResponse::Ok().body("index resource")
}


pub async fn get_user(req: HttpRequest) -> HttpResponse {
    let name = req.match_info().get("name").unwrap_or("World");
    HttpResponse::Ok().body(format!("Hello {}!", name))
}

pub async fn updata_user(req: HttpRequest) -> HttpResponse {
    let name = req.match_info().get("name").unwrap_or("World");
    HttpResponse::Ok().body(format!("Hello {}!", name))
}