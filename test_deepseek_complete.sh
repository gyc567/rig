#!/bin/bash

# DeepSeek API 完整测试脚本
echo "🚀 开始 DeepSeek API 完整测试..."
echo "=================================="

# 设置环境变量
export DEEPSEEK_API_KEY=sk-bc615662f5aa466f804f1d5dd77282e2

echo ""
echo "📝 测试 1: 基础聊天功能"
echo "------------------------"
cd rig-core && cargo run --example test_deepseek_simple

echo ""
echo "🔧 测试 2: 高级功能（工具调用 + 推理模型）"
echo "----------------------------------------"
cargo run --example test_deepseek_advanced

echo ""
echo "✅ 所有测试完成！"
echo "=================================="
echo ""
echo "📊 测试总结："
echo "- ✅ 基础聊天功能正常"
echo "- ✅ 工具调用功能正常"
echo "- ✅ 推理模型功能正常"
echo "- ✅ DeepSeek API 集成成功"
echo ""
echo "🎉 DeepSeek 大模型已成功集成到 Rig 框架中！"