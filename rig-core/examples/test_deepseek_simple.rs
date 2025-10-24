use rig::prelude::*;
use rig::providers::deepseek;
use rig::completion::Prompt;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    println!("🚀 测试 DeepSeek API 接口...\n");

    // 创建 DeepSeek 客户端
    let client = deepseek::Client::from_env();
    
    // 创建一个简单的 Agent
    let agent = client
        .agent(deepseek::DEEPSEEK_CHAT)
        .preamble("你是一个友好的AI助手，请用中文简洁地回答问题。")
        .build();

    // 测试基础对话
    println!("📝 测试基础对话功能:");
    let response = agent.prompt("你好，请简单介绍一下你自己").await?;
    println!("DeepSeek 回复: {}\n", response);

    // 测试数学问题
    println!("🧮 测试数学计算:");
    let math_response = agent.prompt("请计算 25 * 4 等于多少？").await?;
    println!("数学计算结果: {}\n", math_response);

    // 测试创意任务
    println!("🎨 测试创意任务:");
    let creative_response = agent.prompt("请写一句关于人工智能的名言").await?;
    println!("创意回复: {}\n", creative_response);

    println!("✅ DeepSeek API 测试完成！");
    
    Ok(())
}