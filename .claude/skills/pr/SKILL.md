---
name: pr
description: Create a pull request using the project PR template
argument-hint: "[optional description]"
allowed-tools: Bash,Read
---

# Pull Request作成

プロジェクトのPRテンプレートに沿ってPull Requestを作成する。

## Task

1. `.github/PULL_REQUEST_TEMPLATE.md` を読み、テンプレートの構成を把握する
2. `git status` で未コミットの変更がないか確認
3. `git log main...HEAD --oneline` と `git diff main...HEAD` でブランチの全変更を把握
4. テンプレートの各セクションを埋めてPR本文を作成（PRタイトルは英語で書くこと）
5. `git push -u origin <branch>` でリモートにpush
6. `gh pr create` でPRを作成
7. PR URLを返す

Use $ARGUMENTS as additional context if provided.
