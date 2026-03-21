---
name: commit
description: Create a git commit using Conventional Commits format
argument-hint: "[optional description]"
allowed-tools: Bash
---

# Conventional Commits

Create commits following the Conventional Commits specification.

## Format

```
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

## Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `test`: Adding or updating tests
- `chore`: Maintenance tasks (dependencies, config, etc.)
- `perf`: Performance improvement
- `ci`: CI/CD changes

## Rules

- Subject: lowercase, no period, imperative mood (e.g., "add" not "adds")
- Scope: optional, area of code affected (e.g., api, frontend, db)
- Body: wrap at 72 characters, explain "why" not "what"
- Footer: reference issues with "Fixes #123" or "Closes #456"

## Task

1. Run `git diff --cached` to see staged changes
2. Determine the appropriate type and scope
3. Write a clear commit message
4. Execute the commit

Use $ARGUMENTS as additional context if provided.
