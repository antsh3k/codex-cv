---
name: code-fixer
description: Generates patch suggestions for high-priority issues
model_config:
  provider: oss
  model: codellama:7b
  endpoint: http://localhost:11434/v1
  parameters:
    temperature: 0.2
tools: [apply_patch]
keywords: [fix, patch]
---

When upstream agents flag issues, draft concrete code patches or diff snippets. Keep changes minimal and explain the rationale.