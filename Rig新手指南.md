# Rig 新手指南

## 什么是 Rig？

Rig 是一个用 Rust 编写的强大库，专门用于构建基于大语言模型（LLM）的应用程序。它的设计理念是**人体工程学**和**模块化**，让开发者能够轻松地集成各种 AI 模型和向量数据库。

## 核心特性

- 🤖 **智能代理工作流**：支持多轮对话和流式处理
- 🔌 **统一接口**：20+ 模型提供商，10+ 向量存储，全部使用统一的 API
- 🛠️ **工具集成**：支持函数调用和工具使用
- 📊 **RAG 支持**：完整的检索增强生成功能
- 🌐 **WASM 兼容**：核心库支持 WebAssembly
- 📈 **可观测性**：完整的 OpenTelemetry 支持

## 项目结构

```
rig/
├── rig-core/           # 核心库，包含所有基础功能
├── rig-mongodb/        # MongoDB 向量存储集成
├── rig-lancedb/        # LanceDB 向量存储集成
├── rig-qdrant/         # Qdrant 向量存储集成
├── rig-sqlite/         # SQLite 向量存储集成
├── rig-neo4j/          # Neo4j 图数据库集成
├── rig-postgres/       # PostgreSQL 向量存储集成
├── rig-fastembed/      # FastEmbed 嵌入模型集成
├── rig-bedrock/        # AWS Bedrock 集成
└── ...                 # 其他集成包
```

## 快速开始

### 1. 安装

```bash
cargo add rig-core
cargo add tokio --features macros,rt-multi-thread
```

### 2. 基础示例

```rust
use rig::{completion::Prompt, providers::openai};

#[tokio::main]
async fn main() {
    // 创建 OpenAI 客户端（需要设置 OPENAI_API_KEY 环境变量）
    let openai_client = openai::Client::from_env();
    
    // 创建 GPT-4 代理
    let gpt4 = openai_client.agent("gpt-4").build();
    
    // 发送提示并获取响应
    let response = gpt4
        .prompt("你好，请介绍一下你自己")
        .await
        .expect("Failed to prompt GPT-4");
    
    println!("GPT-4: {response}");
}
```

## 核心概念

### 1. 客户端（Client）

每个 AI 提供商都有对应的客户端，用于初始化模型：

```rust
// OpenAI
let openai_client = openai::Client::from_env();

// Anthropic
let anthropic_client = anthropic::Client::from_env();

// Ollama（本地模型）
let ollama_client = ollama::Client::from_url("http://localhost:11434");
```

### 2. 代理（Agent）

代理是 Rig 的核心抽象，它将模型、系统提示、上下文和工具组合在一起：

```rust
let agent = openai_client
    .agent("gpt-4o")
    .preamble("你是一个有用的助手，专门帮助用户解决编程问题。")
    .temperature(0.7)
    .max_tokens(1000)
    .build();
```

### 3. 工具（Tools）

代理可以使用工具来扩展其能力：

```rust
use rig::tool::Tool;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Calculator;

impl Tool for Calculator {
    const NAME: &'static str = "add";
    type Error = anyhow::Error;
    type Args = AddArgs;
    type Output = i32;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "add".to_string(),
            description: "将两个数字相加".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "x": {"type": "number", "description": "第一个数字"},
                    "y": {"type": "number", "description": "第二个数字"}
                },
                "required": ["x", "y"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(args.x + args.y)
    }
}

// 使用工具
let calculator_agent = openai_client
    .agent("gpt-4o")
    .preamble("你是一个计算器助手")
    .tool(Calculator)
    .build();
```

### 4. 嵌入模型（Embeddings）

用于生成文本的向量表示：

```rust
use rig::embeddings::EmbeddingsBuilder;

let embedding_model = openai_client.embedding_model("text-embedding-ada-002");

let embeddings = EmbeddingsBuilder::new(embedding_model)
    .documents(vec!["文档1", "文档2", "文档3"])
    .build()
    .await?;
```

### 5. 向量存储和 RAG

实现检索增强生成：

```rust
use rig::{Embed, vector_store::in_memory_store::InMemoryVectorStore};

#[derive(Embed, Clone, Debug)]
struct Document {
    id: String,
    #[embed]
    content: String,
}

// 创建文档
let documents = vec![
    Document { id: "1".to_string(), content: "Rust 是一种系统编程语言".to_string() },
    Document { id: "2".to_string(), content: "Python 是一种高级编程语言".to_string() },
];

// 生成嵌入
let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
    .documents(documents)
    .build()
    .await?;

// 创建向量存储
let vector_store = InMemoryVectorStore::from_documents(embeddings);
let index = vector_store.index(embedding_model);

// 创建 RAG 代理
let rag_agent = openai_client
    .agent("gpt-4")
    .preamble("你是一个编程助手，使用提供的文档来回答问题")
    .dynamic_context(2, index)  // 动态检索 2 个最相关的文档
    .build();
```

## 支持的提供商

### LLM 提供商
- **OpenAI**: GPT-4, GPT-3.5, DALL-E
- **Anthropic**: Claude 系列
- **Google**: Gemini 系列
- **Cohere**: Command 系列
- **Ollama**: 本地模型支持
- **Groq**: 高速推理
- **Together AI**: 开源模型
- **Hugging Face**: 各种开源模型
- **Azure OpenAI**: 企业级 OpenAI 服务
- 还有更多...

### 向量数据库
- **MongoDB**: 文档数据库 + 向量搜索
- **Qdrant**: 专业向量数据库
- **LanceDB**: 高性能向量数据库
- **SQLite**: 轻量级本地存储
- **PostgreSQL**: 关系数据库 + pgvector
- **Neo4j**: 图数据库 + 向量搜索
- **Milvus**: 云原生向量数据库

## 实际应用示例

### 1. 聊天机器人

```rust
let chatbot = openai_client
    .agent("gpt-4o")
    .preamble("你是一个友好的聊天机器人")
    .build();

let response = chatbot.prompt("今天天气怎么样？").await?;
```

### 2. 代码助手

```rust
let code_assistant = openai_client
    .agent("gpt-4o")
    .preamble("你是一个 Rust 编程专家，帮助用户解决代码问题")
    .build();

let response = code_assistant
    .prompt("如何在 Rust 中处理错误？")
    .await?;
```

### 3. 文档问答系统

```rust
// 加载文档
let documents = load_documents_from_directory("./docs")?;

// 创建 RAG 系统
let qa_system = openai_client
    .agent("gpt-4")
    .preamble("基于提供的文档回答用户问题")
    .dynamic_context(3, document_index)
    .build();

let answer = qa_system
    .prompt("如何使用 Rig 创建代理？")
    .await?;
```

## 高级功能

### 1. 流式响应

```rust
use rig::completion::Chat;

let mut stream = agent.chat_stream("请写一个长故事", vec![]).await?;

while let Some(chunk) = stream.next().await {
    print!("{}", chunk?);
}
```

### 2. 多轮对话

```rust
use rig::completion::message::{Message, MessageRole};

let mut chat_history = vec![
    Message::new(MessageRole::User, "你好".to_string()),
    Message::new(MessageRole::Assistant, "你好！有什么可以帮助你的吗？".to_string()),
];

let response = agent.chat("请解释量子计算", chat_history).await?;
```

### 3. 自定义工具

```rust
#[derive(Deserialize, Serialize)]
struct WeatherTool;

impl Tool for WeatherTool {
    const NAME: &'static str = "get_weather";
    // ... 实现工具逻辑
}
```

## 最佳实践

1. **环境变量管理**: 使用 `.env` 文件管理 API 密钥
2. **错误处理**: 使用 `anyhow` 或 `thiserror` 进行错误处理
3. **异步编程**: 充分利用 Rust 的异步特性
4. **资源管理**: 合理配置模型参数（温度、最大令牌数等）
5. **向量存储选择**: 根据数据规模选择合适的向量数据库

## 调试和监控

Rig 支持完整的可观测性：

```rust
// 启用日志
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();

// OpenTelemetry 集成
let agent = openai_client
    .agent("gpt-4o")
    .build();
```

## 下一步

1. 查看 `rig-core/examples/` 目录中的更多示例
2. 阅读官方文档：https://docs.rig.rs
3. 探索不同的向量数据库集成
4. 尝试构建自己的工具和代理

Rig 的设计让你能够从简单的聊天机器人开始，逐步构建复杂的 AI 应用程序。它的模块化架构意味着你可以根据需要添加功能，而不会被不必要的复杂性所困扰。