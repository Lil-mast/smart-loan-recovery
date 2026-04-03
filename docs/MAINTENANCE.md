# Maintenance

## Main Objective
Sustainable loan recovery platform with low-ops, auditable rules.

## Specific Objectives
1. **System Requirement Research**: Rules-based (evolvable to ML).
2. **Design**: SQLite + Docker.
3. **Test and Implementation**: See testing-implementation.md.
4. **Maintenance** (focus):
   - **DB**: SQLite loans.db/test_loans.db; JSON backups (loans_backup.json).
   - **Config**: .env DATABASE_URL, SESSION_SECRET (src/config.rs).
   - **Deploy**: Dockerfile, fly.toml (`fly deploy`).
   - **Logs**: env_logger in src/main.rs.
   - **Updates**: Edit recovery.rs rules; `cargo update`.
   - **Monitoring**: Actix middleware::Logger.

**Routine**: `cargo check`, backup DB, `fly deploy`. Check TODO.md.
