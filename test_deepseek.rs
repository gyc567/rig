use rig::prelude::*;
use rig::providers::deepseek;
use rig::completion::{Prompt, ToolDefinition};
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    println!("🚀 开始测试 DeepSeek API 接口...\n");

    // 测试 1: 基础聊天功能
    test_basic_chat().await?;
    
    // 测试 2: 工具调用功能
    test_tool_calling().await?;
    
    // 测试 3: 流式响应
    test_streaming().await?;

    println!("✅ 所有测试完成！");
    Ok(())
}

/// 测试基础聊天功能
async fn test_basic_chat() -> Result<(), anyhow::Error> {
    println!("📝 测试 1: 基础聊天功能");
    
    let client = deepseek::Client::from_env();
    let agent = client
        .agent(deepseek::DEEPSEEK_CHAT)
        .preamble("你是一个友好的AI助手，请用中文回答问题。")
        .build();

    let response = agent.prompt("请简单介绍一下你自己").await?;
    println!("DeepSeek 回复: {}\n", response);
    
    Ok(())
}

/// 测试工具调用功能
async fn test_tool_calling() -> Result<(), anyhow::Error> {
    println!("🔧 测试 2: 工具调用功能");
    
    let client = deepseek::Client::from_env();
    let calculator_agent = client
        .agent(deepseek::DEEPSEEK_CHAT)
        .preamble("你是一个计算器助手，使用提供的工具来执行数学运算。")
        .tool(Calculator)
        .build();

    let response = calculator_agent
        .prompt("请计算 123 + 456 的结果")
        .await?;
    
    println!("计算结果: {}\n", response);
    
    Ok(())
}

/// 测试流式响应
async fn test_streaming() -> Result<(), anyhow::Error> {
    println!("🌊 测试 3: 流式响应");
    
    let client = deepseek::Client::from_env();
    let model = client.completion_model(deepseek::DEEPSEEK_CHAT);
    
    let mut stream = model
        .stream(&"请写一首关于人工智能的短诗".into())
        .await?;

    print!("流式输出: ");
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(chunk) => {
                if let Some(content) = chunk.content() {
                    print!("{}", content);
                }
            }
            Err(e) => {
                eprintln!("流式响应错误: {}", e);
                break;
            }
        }
    }
    println!("\n");
    
    Ok(())
}

// 计算器工具定义
#[derive(Deserialize, Serialize)]
struct Calculator;

#[derive(Deserialize)]
struct CalculatorArgs {
    expression: String,
}

#[derive(Debug, thiserror::Error)]
#[error("计算错误")]
struct CalculatorError;

impl Tool for Calculator {
    const NAME: &'static str = "calculator";
    type Error = CalculatorError;
    type Args = CalculatorArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "calculator".to_string(),
            description: "执行基础数学运算，支持加减乘除".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "要计算的数学表达式，例如: '123 + 456'"
                    }
                },
                "required": ["expression"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("[工具调用] 计算表达式: {}", args.expression);
        
        // 简单的计算器实现（仅支持基础运算）
        let result = match evaluate_expression(&args.expression) {
            Ok(result) => result,
            Err(_) => return Err(CalculatorError),
        };
        
        Ok(format!("{} = {}", args.expression, result))
    }
}

// 简单的表达式计算函数
fn evaluate_expression(expr: &str) -> Result<f64, Box<dyn std::error::Error>> {
    let expr = expr.replace(" ", "");
    
    // 支持简单的加法运算
    if let Some(pos) = expr.find('+') {
        let left: f64 = expr[..pos].parse()?;
        let right: f64 = expr[pos + 1..].parse()?;
        return Ok(left + right);
    }
    
    // 支持简单的减法运算
    if let Some(pos) = expr.rfind('-') {
        if pos > 0 { // 确保不是负号
            let left: f64 = expr[..pos].parse()?;
            let right: f64 = expr[pos + 1..].parse()?;
            return Ok(left - right);
        }
    }
    
    // 支持简单的乘法运算
    if let Some(pos) = expr.find('*') {
        let left: f64 = expr[..pos].parse()?;
        let right: f64 = expr[pos + 1..].parse()?;
        return Ok(left * right);
    }
    
    // 支持简单的除法运算
    if let Some(pos) = expr.find('/') {
        let left: f64 = expr[..pos].parse()?;
        let right: f64 = expr[pos + 1..].parse()?;
        if right != 0.0 {
            return Ok(left / right);
        }
    }
    
    // 如果没有运算符，尝试解析为数字
    Ok(expr.parse()?)
}