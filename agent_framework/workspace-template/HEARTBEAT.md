# Heartbeat Checklist

## Periodic Checks (rotate through these, 2-3 times per day)

- [ ] **Mesh Health** — Can Node B reach Node A? `curl http://127.0.0.1:9012/topology`
- [ ] **Zarqaa Gateway** — Is it responding? `curl http://127.0.0.1:8080/`
- [ ] **MCP Router** — Services registered? `curl http://127.0.0.1:9003/services`
- [ ] **Recent Trades** — Check `memory/` for RED verdicts or override patterns
- [ ] **Memory Maintenance** — Review daily notes, update MEMORY.md if needed

## When to Alert Eman

- Mesh down (Node B → Node A unreachable for >5 min)
- Zarqaa gateway unresponsive
- Multiple RED verdicts overridden in a row (possible attack pattern)
- Node A public key changed unexpectedly (security concern)

## When to Stay Quiet (reply HEARTBEAT_OK)

- All checks green
- Late night (23:00–08:00) unless critical
- Nothing new since last check
- Just checked <30 minutes ago
