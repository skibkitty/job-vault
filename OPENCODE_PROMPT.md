# Recommended OpenCode Prompts

## Find the next task

```text
Read AGENTS.md, project/CURRENT_STATE.md, project/HANDOFF.md, and project/TASKS.md. Determine the highest-priority READY task whose dependencies are complete. Explain why it is the next task, what it requires, and any questions or blockers before implementation.
```

## Implement the next task

```text
Read AGENTS.md, project/CURRENT_STATE.md, project/HANDOFF.md, project/TASKS.md, and all relevant architecture/security documents and ADRs.

Determine the highest-priority READY task whose dependencies are complete.

Implement that task only. Ask me questions if you need my input, if a required decision is ambiguous, if a security-sensitive decision cannot be made safely from the documentation, or if something is genuinely not possible in the current environment.

Do not invent requirements. Do not weaken security. Do not add network access, telemetry, permissions, cloud services, remote AI, or unrelated features without explicit authorization.

Run all relevant tests and checks. Review your Git diff.

Before finishing:
- update the task status;
- update project/CURRENT_STATE.md;
- update project/HANDOFF.md;
- update relevant documentation;
- add/update an ADR if an architectural decision changed;
- update project/SECURITY_LOG.md for security-relevant changes;
- create a logical Git commit.

If blocked, do not pretend the task is complete. Document the blocker and tell me exactly what input or action is required.
```

## Resume after interruption

```text
Read AGENTS.md, project/CURRENT_STATE.md, project/HANDOFF.md, and project/TASKS.md. Determine what the previous agent was doing, verify the repository state, run the relevant tests, and either continue the current task or explain why it must be blocked. Do not rely on prior conversation history.
```

## Security review

```text
Perform a security review of the current implementation using AGENTS.md, docs/SECURITY.md, docs/PRIVACY.md, and docs/THREAT-MODEL.md. Do not modify code yet. Report findings by severity, identify concrete files/locations, and create recommended follow-up TASK entries for issues that require implementation.
```
