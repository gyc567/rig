# RIG新手入门手册

## 目录
1. [项目简介](#项目简介)
2. [核心概念](#核心概念)
3. [环境准备](#环境准备)
4. [项目结构](#项目结构)
5. [快速开始](#快速开始)
6. [核心模块详解](#核心模块详解)
7. [实际应用示例](#实际应用示例)
8. [进阶内容](#进阶内容)
9. [常见问题](#常见问题)

## 项目简介

Rig是一个用Rust编写的AI应用开发库，专注于提供简单易用且模块化的接口来构建基于大语言模型(LLM)的应用程序。

### 主要特性
- 支持多种LLM提供商（OpenAI、Anthropic、Gemini等20+种）
- 支持多种向量存储（MongoDB、LanceDB、Qdrant等10+种）
- 提供Agent工作流，支持多轮对话和流式处理
- 完整支持LLM补全和嵌入工作流
- 支持转录、音频生成和图像生成模型功能
- 全WASM兼容（仅限核心库）

## 核心概念

### Completion和Embedding模型
Rig为LLM和嵌入模型提供了统一的API。每个提供商（如OpenAI、Cohere）都有一个[Client](file:///Users/guoyingcheng/claude_pro/rig/rig-core/src/client/mod.rs#L115-L115)结构体，可用于初始化补全和嵌入模型。

### Agents
Rig提供了高级抽象的[Agent](file:///Users/guoyingcheng/claude_pro/rig/rig-core/src/agent/completion.rs#L12-L12)类型，可以用于创建从简单bot到完整RAG系统的各种应用。

### 向量存储和索引
Rig提供了向量存储和索引的通用接口，可以作为RAG系统的知识库或自定义架构中的上下文文档源。

## 环境准备

### 系统要求
- Rust 1.70或更高版本
- Cargo包管理器
- Git版本控制

### 安装Rust
```bash
# 使用rustup安装Rust（推荐）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 验证安装
rustc --version
cargo --version
```

### 克隆项目
```bash
git clone https://github.com/0xPlaygrounds/rig.git
cd rig
```

### 安装依赖
```bash
# 安装项目依赖
cargo build
```

## 项目结构

```
rig/
├── rig-core/              # 核心库
│   ├── src/               # 源代码
│   │   ├── agent/         # Agent相关实现
│   │   ├── completion/    # 补全模型相关
│   │   ├── embeddings/    # 嵌入模型相关
│   │   ├── providers/     # 各种LLM提供商实现
│   │   ├── vector_store/  # 向量存储实现
│   │   └── ...
│   ├── examples/          # 示例代码
│   └── ...
├── rig-lancedb/           # LanceDB向量存储集成
├── rig-mongodb/           # MongoDB向量存储集成
├── rig-qdrant/            # Qdrant向量存储集成
└── ...                    # 其他向量存储和模型提供商集成
```

## 快速开始

### 简单示例
创建一个简单的OpenAI聊天bot：

```rust
use rig::{completion::Prompt, providers::openai};

#[tokio::main]
async fn main() {
    // 创建OpenAI客户端
    // 需要设置`OPENAI_API_KEY`环境变量
    let openai_client = openai::Client::from_env();

    let gpt4 = openai_client.agent("gpt-4").build();

    // 向模型提问并打印响应
    let response = gpt4
        .prompt("Who are you?")
        .await
        .expect("Failed to prompt GPT-4");

    println!("GPT-4: {response}");
}
```

添加必要的依赖到Cargo.toml：
```toml
[dependencies]
rig-core = "0.22"
tokio = { version = "1", features = ["full"] }
```

### 运行示例
```bash
# 设置API密钥
export OPENAI_API_KEY=your-api-key

# 运行示例
cargo run --example agent
```

## 核心模块详解

### 1. Agent模块
[Agent](file:///Users/guoyingcheng/claude_pro/rig/rig-core/src/agent/completion.rs#L12-L12)是Rig的核心抽象，结合了LLM模型、系统提示、上下文文档和工具。

#### 创建基本Agent
```rust
use rig::prelude::*;
use rig::{completion::Prompt, providers};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // 创建OpenAI客户端
    let client = providers::openai::Client::from_env();

    // 创建带系统提示的Agent
    let comedian_agent = client
        .agent("gpt-4o")
        .preamble("You are a comedian here to entertain the user using humour and jokes.")
        .build();

    // 向Agent提问并打印响应
    let response = comedian_agent.prompt("Entertain me!").await?;
    println!("{response}");

    Ok(())
}
```

#### 带工具的Agent
```rust
use anyhow::Result;
use rig::prelude::*;
use rig::{
    completion::{Prompt, ToolDefinition},
    providers,
    tool::Tool,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize)]
struct OperationArgs {
    x: i32,
    y: i32,
}

#[derive(Debug, thiserror::Error)]
#[error("Math error")]
struct MathError;

#[derive(Deserialize, Serialize)]
struct Adder;
impl Tool for Adder {
    const NAME: &'static str = "add";
    type Error = MathError;
    type Args = OperationArgs;
    type Output = i32;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "add".to_string(),
            description: "Add x and y together".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "x": {
                        "type": "number",
                        "description": "The first number to add"
                    },
                    "y": {
                        "type": "number",
                        "description": "The second number to add"
                    }
                },
                "required": ["x", "y"],
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("[tool-call] Adding {} and {}", args.x, args.y);
        let result = args.x + args.y;
        Ok(result)
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // 创建OpenAI客户端
    let openai_client = providers::openai::Client::from_env();

    // 创建带工具的Agent
    let calculator_agent = openai_client
        .agent(providers::openai::GPT_4O)
        .preamble("You are a calculator here to help the user perform arithmetic operations. Use the tools provided to answer the user's question.")
        .max_tokens(1024)
        .tool(Adder)
        .build();

    // 使用Agent进行计算
    println!("Calculate 2 - 5");
    println!(
        "OpenAI Calculator Agent: {}",
        calculator_agent.prompt("Calculate 2 - 5").await?
    );

    Ok(())
}
```

### 2. RAG (Retrieval-Augmented Generation)模块
RAG允许Agent基于知识库回答问题。

```rust
use rig::prelude::*;
use rig::providers::openai::client::Client;
use rig::{
    Embed, completion::Prompt, embeddings::EmbeddingsBuilder,
    providers::openai::TEXT_EMBEDDING_ADA_002, vector_store::in_memory_store::InMemoryVectorStore,
};
use serde::Serialize;
use std::vec;

// 要进行RAG的数据
#[derive(Embed, Serialize, Clone, Debug, Eq, PartialEq, Default)]
struct WordDefinition {
    id: String,
    word: String,
    #[embed]
    definitions: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // 创建OpenAI客户端
    let openai_client = Client::from_env();
    let embedding_model = openai_client.embedding_model(TEXT_EMBEDDING_ADA_002);

    // 为文档生成嵌入
    let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
        .documents(vec![
            WordDefinition {
                id: "doc0".to_string(),
                word: "flurbo".to_string(),
                definitions: vec![
                    "1. *flurbo* (name): A flurbo is a green alien that lives on cold planets.".to_string(),
                    "2. *flurbo* (name): A fictional digital currency that originated in the animated series Rick and Morty.".to_string()
                ]
            },
            WordDefinition {
                id: "doc1".to_string(),
                word: "glarb-glarb".to_string(),
                definitions: vec![
                    "1. *glarb-glarb* (noun): A glarb-glarb is a ancient tool used by the ancestors of the inhabitants of planet Jiro to farm the land.".to_string(),
                    "2. *glarb-glarb* (noun): A fictional creature found in the distant, swampy marshlands of the planet Glibbo in the Andromeda galaxy.".to_string()
                ]
            },
        ])?
        .build()
        .await?;

    // 创建向量存储
    let vector_store = InMemoryVectorStore::from_documents(embeddings);

    // 创建向量存储索引
    let index = vector_store.index(embedding_model);
    let rag_agent = openai_client.agent("gpt-4")
        .preamble("
            You are a dictionary assistant here to assist the user in understanding the meaning of words.
            You will find additional non-standard word definitions that could be useful below.
        ")
        .dynamic_context(1, index)
        .build();

    // 向Agent提问并打印响应
    let response = rag_agent.prompt("What does \"glarb-glarb\" mean?").await?;
    println!("{response}");

    Ok(())
}
```

### 3. 向量存储模块
Rig支持多种向量存储后端，如LanceDB、MongoDB、Qdrant等。

#### LanceDB示例
```rust
use std::sync::Arc;
use arrow_array::RecordBatchIterator;
use lancedb::index::vector::IvfPqIndexBuilder;
use rig::{
    embeddings::{EmbeddingModel, EmbeddingsBuilder},
    providers::openai::{Client, TEXT_EMBEDDING_ADA_002},
    vector_store::VectorStoreIndex,
};
use rig_lancedb::{LanceDbVectorIndex, SearchParams};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // 初始化OpenAI客户端
    let openai_client = Client::from_env();
    let model = openai_client.embedding_model(TEXT_EMBEDDING_ADA_002);

    // 初始化本地LanceDB
    let db = lancedb::connect("data/lancedb-store").execute().await?;

    // 创建向量存储索引
    let table = db.open_table("definitions").execute().await?;
    let search_params = SearchParams::default();
    let vector_store_index = LanceDbVectorIndex::new(table, model, "id", search_params).await?;

    // 查询索引
    let results = vector_store_index
        .top_n::<String>("My boss says I zindle too much, what does that mean?", 1)
        .await?;

    println!("Results: {results:?}");
    Ok(())
}
```

## 实际应用示例

### 1. 创建一个文档问答系统
```rust
// 查看完整示例: rig-core/examples/rag.rs
```

### 2. 创建一个多工具Agent
```rust
// 查看完整示例: rig-core/examples/agent_with_tools.rs
```

### 3. 创建一个流式聊天bot
```rust
// 查看完整示例: rig-core/examples/openai_streaming.rs
```

## 进阶内容

### 1. 自定义模型提供商
实现[CompletionModel](file:///Users/guoyingcheng/claude_pro/rig/rig-core/src/completion/request.rs#L345-L345)和[EmbeddingModel](file:///Users/guoyingcheng/claude_pro/rig/rig-core/src/embeddings/embedding.rs#L56-L56)特质来添加新的模型提供商。

### 2. 自定义向量存储
实现[VectorStoreIndex](file:///Users/guoyingcheng/claude_pro/rig/rig-core/src/vector_store/mod.rs#L47-L47)特质来添加新的向量存储。

### 3. 多Agent系统
使用多个Agent协同工作来解决复杂任务。

## 常见问题

### 1. 如何设置API密钥？
大多数提供商需要API密钥，可以通过环境变量设置：
```bash
export OPENAI_API_KEY=your-openai-api-key
export ANTHROPIC_API_KEY=your-anthropic-api-key
```

### 2. 如何选择合适的模型？
- 对于简单任务：使用gpt-3.5-turbo或类似模型
- 对于复杂任务：使用gpt-4或类似模型
- 对于嵌入：使用text-embedding-ada-002或类似模型

### 3. 如何处理错误？
Rig使用`anyhow`和`thiserror`进行错误处理，确保正确处理所有可能的错误情况。

### 4. 如何调试Agent行为？
使用`tracing` crate启用日志记录：
```rust
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

## 更多资源

- [官方文档](https://docs.rig.rs)
- [API参考](https://docs.rs/rig-core/latest/rig/)
- [GitHub仓库](https://github.com/0xPlaygrounds/rig)
- [示例代码](./rig-core/examples/)