# Security Policy

## Supported Versions

Security fixes target the latest released version of Aura Translation.

## Reporting a Vulnerability

Please do not report suspected vulnerabilities through a public issue.

Use GitHub's private vulnerability reporting or open a private security advisory for this repository. Include:

- Affected version or commit.
- Operating system and install method.
- Steps to reproduce.
- Impact and any known workaround.

## Secrets and Local Data

Aura Translation stores user preferences and recent translation history locally. API keys default to the OS credential store on supported desktop builds, with plaintext config storage available only as an explicit fallback.

Never include real provider API keys, credential-store dumps, local config files, or private clipboard contents in public reports.
