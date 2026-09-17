# Provider Configuration / Provider 配置

ContextLab's model gateway is the boundary between core Context workflows and external model providers. The first implementation focuses on safe configuration, predictable defaults, and secret redaction.

ContextLab 的 model gateway 是核心 Context 工作流与外部模型 provider 之间的边界。第一版聚焦安全配置、可预测默认值和密钥脱敏。

## Environment Variables / 环境变量

```bash
DEEPSEEK_API_BASE_URL=https://api.deepseek.com
DEEPSEEK_API_KEY=
OPENAI_COMPAT_API_BASE_URL=https://codex.hiyo.top
OPENAI_COMPAT_API_KEY=
```

Real API keys belong only in local `.env` files. `.env.example` documents names and defaults, but must never contain real secrets.

真实 API key 只应放在本地 `.env` 文件里。`.env.example` 只记录变量名和默认端点，绝不能包含真实密钥。

## Public API / 公开 API

`GET /api/v1/providers` returns provider readiness without exposing secrets:

`GET /api/v1/providers` 返回 provider 就绪状态，但不会暴露密钥：

```json
{
  "providers": [
    {
      "id": "deepseek",
      "display_name": "DeepSeek",
      "configured": true,
      "base_url": "https://api.deepseek.com/",
      "api_key_env": "DEEPSEEK_API_KEY",
      "api_key_fingerprint": "sk-...abcd"
    }
  ]
}
```

The fingerprint is only for local diagnostics. It is not an authentication credential.

指纹只用于本地诊断，不是认证凭据。
