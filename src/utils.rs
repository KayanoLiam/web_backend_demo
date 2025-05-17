// 外部库导入
use actix_web::web;                  // 用于Web相关工具和类型
use futures::stream::{self, Stream};  // 用于创建和操作异步流
use futures::StreamExt;               // 提供流的扩展方法，如take()
use std::time::Duration;              // 用于表示时间段
use tokio::time::interval;            // 用于创建定时器

// 内部模块导入
use crate::errors::InternalDbError;   // 自定义数据库错误类型

/// 创建服务器发送事件(SSE)流
///
/// 生成一个每秒发送一次递增计数器值的流
/// 用于实现实时更新功能
///
/// # 返回值
/// * 返回一个异步流，每秒产生一个包含计数器值的SSE格式消息
///   流限制为最多产生10条消息
pub fn create_sse_stream() -> impl Stream<Item = Result<web::Bytes, std::io::Error>> {
    // 定义一个计数器，用于生成递增的数据
    let mut counter: usize = 0;

    // 创建一个定时器，每隔1秒触发一次
    let mut interval = interval(Duration::from_secs(1));

    // 使用poll_fn创建一个自定义流
    // 这个流会在每次被轮询时检查定时器是否触发
    stream::poll_fn(move |cx| match interval.poll_tick(cx) {
        // 如果定时器触发（每秒一次）
        std::task::Poll::Ready(_) => {
            counter += 1;  // 计数器加1

            // 构造SSE格式的数据字符串
            // SSE格式要求每条消息以"data: "开头，以两个换行符结束
            let msg = format!("data: {}\n\n", counter);

            // 返回数据
            // 将字符串转换为字节流并包装在Result和Option中
            std::task::Poll::Ready(Some(Ok(web::Bytes::from(msg))))
        }
        // 如果定时器还没到时间，则挂起等待
        // 这会告诉运行时当前没有数据可用，稍后再检查
        std::task::Poll::Pending => std::task::Poll::Pending,
    })
    // 限制流最多产生10条消息
    // 达到限制后流会自动结束
    .take(10)
}

/// 模拟可能失败的操作
///
/// 这个函数总是返回错误，用于演示错误处理
/// 在实际应用中，这里可能是数据库操作或其他可能失败的IO操作
///
/// # 返回值
/// * 返回Result类型，总是包含InternalDbError错误
pub fn do_thing_that_may_fail() -> Result<(), InternalDbError> {
    // 在实际应用中，这里会有一些可能会失败的操作
    // 例如数据库查询、文件操作或网络请求

    // 为了演示目的，这个函数总是返回错误
    // 在实际应用中，应该根据操作结果返回Ok或Err
    Err(InternalDbError)  // 返回一个InternalDbError错误
}
