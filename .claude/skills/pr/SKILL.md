---
name: pr
description: Create a pull request using the project PR template
argument-hint: "[optional description]"
allowed-tools: Bash,Read
---

# Create Pull Request

Create a pull request following the project's PR template.

## Task

1. Read `.github/PULL_REQUEST_TEMPLATE.md` to understand the template structure
2. Run `git status` to check for uncommitted changes
3. Run `git log main...HEAD --oneline` and `git diff main...HEAD` to review all branch changes
4. Fill in each section of the template to compose the PR body (write the PR title and body in English)
5. Run `git push -u origin <branch>` to push to remote
6. Run `gh pr create` to create the PR
7. Return the PR URL

Use $ARGUMENTS as additional context if provided.
