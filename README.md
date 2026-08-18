# Job Vault — OpenCode Project Package

This repository specification is for a private, personal-use Chrome extension + local companion application that tracks job postings, stores historical snapshots, compares reposts, and highlights meaningful changes.

## How to use this package

Give the entire `job-vault-opencode-spec` directory to OpenCode as the project documentation, then use short prompts such as:

> Read the project instructions and tell me the next task.

and:

> Implement the next task. Ask me questions if you need my input or if something is not possible for you to do.

OpenCode must treat the repository's project-management files as the source of truth, not prior conversation history.

## Important

This project does **not** claim HIPAA or GLBA legal compliance. It uses technical safeguards inspired by those frameworks because the application handles highly sensitive personal job-search data.

## Current phase

Planning / pre-implementation.

The first implementation task is deliberately project infrastructure and documentation, not the job tracker itself.

## Recommended workflow

1. Initialize a Git repository.
2. Copy these files into the repository.
3. Have OpenCode complete the first READY task.
4. Review security-sensitive decisions yourself.
5. Continue asking OpenCode for the next task.
6. Before ending a session, OpenCode must update the project state and handoff files.
