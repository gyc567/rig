use rig::prelude::*;
use rig::providers::deepseek;
use rig::completion::{Prompt, ToolDefinition, CompletionModel};
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    println!("🚀 DeepSeek 高级功能测试...\n");

    // 测试 1: 工具调用功能
    test_tool_calling().await?;
    
    // 测试 2: 推理模型
    test_reasoning_model().await?;

    println!("✅ 所有高级测试完成！");
    Ok(())
}

/// 测试工具调用功能
async fn test_tool_calling() -> Result<(), anyhow::Error> {
    println!("🔧 测试工具调用功能:");
    
    let client = deepseek::Client::from_env();
    let calculator_agent = client
        .agent(deepseek::DEEPSEEK_CHAT)
        .preamble("你是一个数学助手，使用提供的工具来执行精确的数学运算。")
        .tool(Calculator)
        .tool(WeatherTool)
        .build();

    let response = calculator_agent
        .prompt("请计算 (15 + 25) * 2 的结果，然后告诉我北京的天气如何")
        .await?;
    
    println!("工具调用结果: {}\n", response);
    
    Ok(())
}



/// 测试推理模型
async fn test_reasoning_model() -> Result<(), anyhow::Error> {
    println!("🧠 测试推理模型:");
    
    let client = deepseek::Client::from_env();
    let reasoning_agent = client
        .agent(deepseek::DEEPSEEK_REASONER)
        .preamble("你是一个逻辑推理专家，请仔细分析问题并给出详细的推理过程。")
        .build();

    let response = reasoning_agent
        .prompt("有三个盒子，一个装金子，一个装银子，一个是空的。每个盒子上都有标签，但所有标签都是错的。如果我从标着'金子'的盒子里拿出一个银子，那么金子在哪个盒子里？")
        .await?;
    
    println!("推理结果: {}\n", response);
    
    Ok(())
}

// 计算器工具
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
            description: "执行数学运算，支持基础的加减乘除运算".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "要计算的数学表达式，例如: '(15 + 25) * 2'"
                    }
                },
                "required": ["expression"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("[工具调用] 计算表达式: {}", args.expression);
        
        // 简单的表达式计算（实际项目中可以使用更强大的表达式解析器）
        let result = match evaluate_simple_expression(&args.expression) {
            Ok(result) => result,
            Err(_) => return Err(CalculatorError),
        };
        
        Ok(format!("{} = {}", args.expression, result))
    }
}

// 天气工具（模拟）
#[derive(Deserialize, Serialize)]
struct WeatherTool;

#[derive(Deserialize)]
struct WeatherArgs {
    city: String,
}

#[derive(Debug, thiserror::Error)]
#[error("天气查询错误")]
struct WeatherError;

impl Tool for WeatherTool {
    const NAME: &'static str = "get_weather";
    type Error = WeatherError;
    type Args = WeatherArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "get_weather".to_string(),
            description: "获取指定城市的天气信息".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "city": {
                        "type": "string",
                        "description": "要查询天气的城市名称"
                    }
                },
                "required": ["city"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("[工具调用] 查询城市天气: {}", args.city);
        
        // 模拟天气数据
        let weather_data = match args.city.as_str() {
            "北京" => "北京今天晴朗，气温15-25°C，微风",
            "上海" => "上海今天多云，气温18-28°C，东南风",
            "深圳" => "深圳今天阵雨，气温22-30°C，南风",
            _ => "抱歉，暂时无法获取该城市的天气信息",
        };
        
        Ok(weather_data.to_string())
    }
}

// 简单的数学表达式计算
fn evaluate_simple_expression(expr: &str) -> Result<f64, Box<dyn std::error::Error>> {
    let expr = expr.replace(" ", "").replace("(", "").replace(")", "");
    
    // 处理乘法优先级：先找乘法
    if let Some(pos) = expr.find('*') {
        let left_part = &expr[..pos];
        let right_part = &expr[pos + 1..];
        
        // 如果左边有加法，先计算加法
        if let Some(add_pos) = left_part.find('+') {
            let a: f64 = left_part[..add_pos].parse()?;
            let b: f64 = left_part[add_pos + 1..].parse()?;
            let c: f64 = right_part.parse()?;
            return Ok((a + b) * c);
        } else {
            let left: f64 = left_part.parse()?;
            let right: f64 = right_part.parse()?;
            return Ok(left * right);
        }
    }
    
    // 处理加法
    if let Some(pos) = expr.find('+') {
        let left: f64 = expr[..pos].parse()?;
        let right: f64 = expr[pos + 1..].parse()?;
        return Ok(left + right);
    }
    
    // 处理减法
    if let Some(pos) = expr.rfind('-') {
        if pos > 0 {
            let left: f64 = expr[..pos].parse()?;
            let right: f64 = expr[pos + 1..].parse()?;
            return Ok(left - right);
        }
    }
    
    // 处理除法
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