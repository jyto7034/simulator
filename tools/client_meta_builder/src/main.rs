use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::Parser;
use serde::Deserialize;
use sqlx::{sqlite::SqliteConnectOptions, Connection, SqliteConnection};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(name = "client_meta_builder")]
#[command(about = "Build client_meta.db (SQLite) from game_resources_client/meta.ron")]
struct Args {
    /// Input RON file.
    #[arg(long, default_value = "game_resources_client/meta.ron")]
    input: PathBuf,

    /// Output SQLite DB path.
    #[arg(long, default_value = "game_resources_client/client_meta.db")]
    output: PathBuf,

    /// Overwrite output DB if it exists.
    #[arg(long, default_value_t = true)]
    overwrite: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClientMetaDb {
    version: String,

    #[serde(default)]
    units: Vec<UnitMeta>,
    #[serde(default)]
    skills: Vec<SkillMeta>,
    #[serde(default)]
    buffs: Vec<BuffMeta>,
    #[serde(default)]
    equipments: Vec<SimpleItemMeta>,
    #[serde(default)]
    artifacts: Vec<SimpleItemMeta>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UnitMeta {
    base_uuid: Uuid,
    prefab: String,

    #[serde(default)]
    spawn_vfx: Option<String>,
    #[serde(default)]
    death_vfx: Option<String>,

    #[serde(default)]
    basic_projectile: Option<String>,
    #[serde(default)]
    basic_hit_vfx: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SkillMeta {
    skill_id: String,

    #[serde(default)]
    cast_anim: Option<String>,
    #[serde(default)]
    cast_vfx: Option<String>,
    #[serde(default)]
    projectile: Option<String>,
    #[serde(default)]
    hit_vfx: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BuffMeta {
    buff_key: String,

    #[serde(default)]
    apply_vfx: Option<String>,
    #[serde(default)]
    tick_vfx: Option<String>,
    #[serde(default)]
    aura_vfx: Option<String>,
    #[serde(default)]
    expire_vfx: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SimpleItemMeta {
    base_uuid: Uuid,

    #[serde(default)]
    prefab: Option<String>,
    #[serde(default)]
    spawn_vfx: Option<String>,
    #[serde(default)]
    icon: Option<String>,
}

fn ensure_non_empty(label: &str, v: &str) -> Result<()> {
    if v.trim().is_empty() {
        bail!("{label} must be non-empty");
    }
    Ok(())
}

fn ensure_unique<'a>(
    kind: &str,
    key: &str,
    seen: &mut std::collections::HashSet<String>,
) -> Result<()> {
    if !seen.insert(key.to_string()) {
        bail!("duplicate {kind} key: {key}");
    }
    Ok(())
}

fn canonical_uuid(u: Uuid) -> String {
    u.to_string()
}

fn find_repo_root(mut dir: PathBuf) -> Option<PathBuf> {
    loop {
        if dir.join("Cargo.toml").is_file() && dir.join("game_resources_client").is_dir() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn is_repo_scoped_default(path: &Path) -> bool {
    matches!(path.components().next(), Some(Component::Normal(first)) if first == "game_resources_client")
}

fn resolve_input_path(cwd: &Path, repo_root: &Path, input: &Path) -> PathBuf {
    if input.is_absolute() {
        return input.to_path_buf();
    }
    let cwd_rel = cwd.join(input);
    if cwd_rel.is_file() {
        return cwd_rel;
    }
    let root_rel = repo_root.join(input);
    if root_rel.is_file() {
        return root_rel;
    }
    cwd_rel
}

fn resolve_output_path(cwd: &Path, repo_root: &Path, output: &Path) -> PathBuf {
    if output.is_absolute() {
        return output.to_path_buf();
    }
    if is_repo_scoped_default(output) {
        return repo_root.join(output);
    }
    cwd.join(output)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let cwd = std::env::current_dir().context("failed to get current working directory")?;
    let repo_root = find_repo_root(cwd.clone()).unwrap_or_else(|| cwd.clone());

    let input_path = resolve_input_path(&cwd, &repo_root, &args.input);
    let output_path = resolve_output_path(&cwd, &repo_root, &args.output);

    let ron_text = std::fs::read_to_string(&input_path).with_context(|| {
        format!(
            "failed to read input RON: {} (cwd: {}, repo_root: {})",
            input_path.display(),
            cwd.display(),
            repo_root.display()
        )
    })?;

    let meta: ClientMetaDb = ron::de::from_str(&ron_text)
        .with_context(|| format!("failed to parse RON: {}", input_path.display()))?;

    if output_path.exists() {
        if args.overwrite {
            std::fs::remove_file(&output_path).with_context(|| {
                format!("failed to remove existing db: {}", output_path.display())
            })?;
        } else {
            bail!(
                "output already exists (pass --overwrite): {}",
                output_path.display()
            );
        }
    }

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output dir: {}", parent.display()))?;
    }

    validate(&meta)?;

    let options = SqliteConnectOptions::new()
        .filename(&output_path)
        .create_if_missing(true);

    let mut conn = SqliteConnection::connect_with(&options)
        .await
        .with_context(|| format!("failed to open sqlite db: {}", output_path.display()))?;

    create_schema(&mut conn).await?;
    populate(&mut conn, &meta).await?;

    println!("Wrote: {}", output_path.display());
    Ok(())
}

fn validate(meta: &ClientMetaDb) -> Result<()> {
    ensure_non_empty("version", &meta.version)?;

    let mut seen_units = std::collections::HashSet::new();
    for u in &meta.units {
        ensure_unique(
            "unit base_uuid",
            &canonical_uuid(u.base_uuid),
            &mut seen_units,
        )?;
        ensure_non_empty("unit.prefab", &u.prefab)?;
    }

    let mut seen_skills = std::collections::HashSet::new();
    for s in &meta.skills {
        ensure_non_empty("skill.skill_id", &s.skill_id)?;
        ensure_unique("skill_id", &s.skill_id, &mut seen_skills)?;
    }

    let mut seen_buffs = std::collections::HashSet::new();
    for b in &meta.buffs {
        ensure_non_empty("buff.buff_key", &b.buff_key)?;
        ensure_unique("buff_key", &b.buff_key, &mut seen_buffs)?;
    }

    let mut seen_equip = std::collections::HashSet::new();
    for e in &meta.equipments {
        ensure_unique(
            "equipment base_uuid",
            &canonical_uuid(e.base_uuid),
            &mut seen_equip,
        )?;
    }

    let mut seen_art = std::collections::HashSet::new();
    for a in &meta.artifacts {
        ensure_unique(
            "artifact base_uuid",
            &canonical_uuid(a.base_uuid),
            &mut seen_art,
        )?;
    }

    Ok(())
}

async fn create_schema(conn: &mut SqliteConnection) -> Result<()> {
    // Keep schema minimal and query-friendly: PK lookups only.
    let schema = [
        r#"CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL)"#,
        r#"CREATE TABLE units (
            base_uuid TEXT PRIMARY KEY,
            prefab TEXT NOT NULL,
            spawn_vfx TEXT,
            death_vfx TEXT,
            basic_projectile TEXT,
            basic_hit_vfx TEXT
        )"#,
        r#"CREATE TABLE skills (
            skill_id TEXT PRIMARY KEY,
            cast_anim TEXT,
            cast_vfx TEXT,
            projectile TEXT,
            hit_vfx TEXT
        )"#,
        r#"CREATE TABLE buffs (
            buff_key TEXT PRIMARY KEY,
            apply_vfx TEXT,
            tick_vfx TEXT,
            aura_vfx TEXT,
            expire_vfx TEXT
        )"#,
        r#"CREATE TABLE equipments (
            base_uuid TEXT PRIMARY KEY,
            prefab TEXT,
            spawn_vfx TEXT,
            icon TEXT
        )"#,
        r#"CREATE TABLE artifacts (
            base_uuid TEXT PRIMARY KEY,
            prefab TEXT,
            spawn_vfx TEXT,
            icon TEXT
        )"#,
    ];

    for stmt in schema {
        sqlx::query(stmt).execute(&mut *conn).await?;
    }

    Ok(())
}

async fn populate(conn: &mut SqliteConnection, meta: &ClientMetaDb) -> Result<()> {
    let mut tx = conn.begin().await?;

    sqlx::query("INSERT INTO meta(key, value) VALUES(?, ?)")
        .bind("schema_version")
        .bind("1")
        .execute(&mut *tx)
        .await?;

    sqlx::query("INSERT INTO meta(key, value) VALUES(?, ?)")
        .bind("content_version")
        .bind(&meta.version)
        .execute(&mut *tx)
        .await?;

    for u in &meta.units {
        sqlx::query(
            "INSERT INTO units(base_uuid, prefab, spawn_vfx, death_vfx, basic_projectile, basic_hit_vfx) VALUES(?, ?, ?, ?, ?, ?)"
        )
        .bind(canonical_uuid(u.base_uuid))
        .bind(&u.prefab)
        .bind(u.spawn_vfx.as_deref())
        .bind(u.death_vfx.as_deref())
        .bind(u.basic_projectile.as_deref())
        .bind(u.basic_hit_vfx.as_deref())
        .execute(&mut *tx)
        .await?;
    }

    for s in &meta.skills {
        sqlx::query(
            "INSERT INTO skills(skill_id, cast_anim, cast_vfx, projectile, hit_vfx) VALUES(?, ?, ?, ?, ?)"
        )
        .bind(&s.skill_id)
        .bind(s.cast_anim.as_deref())
        .bind(s.cast_vfx.as_deref())
        .bind(s.projectile.as_deref())
        .bind(s.hit_vfx.as_deref())
        .execute(&mut *tx)
        .await?;
    }

    for b in &meta.buffs {
        sqlx::query(
            "INSERT INTO buffs(buff_key, apply_vfx, tick_vfx, aura_vfx, expire_vfx) VALUES(?, ?, ?, ?, ?)"
        )
        .bind(&b.buff_key)
        .bind(b.apply_vfx.as_deref())
        .bind(b.tick_vfx.as_deref())
        .bind(b.aura_vfx.as_deref())
        .bind(b.expire_vfx.as_deref())
        .execute(&mut *tx)
        .await?;
    }

    for e in &meta.equipments {
        sqlx::query(
            "INSERT INTO equipments(base_uuid, prefab, spawn_vfx, icon) VALUES(?, ?, ?, ?)",
        )
        .bind(canonical_uuid(e.base_uuid))
        .bind(e.prefab.as_deref())
        .bind(e.spawn_vfx.as_deref())
        .bind(e.icon.as_deref())
        .execute(&mut *tx)
        .await?;
    }

    for a in &meta.artifacts {
        sqlx::query("INSERT INTO artifacts(base_uuid, prefab, spawn_vfx, icon) VALUES(?, ?, ?, ?)")
            .bind(canonical_uuid(a.base_uuid))
            .bind(a.prefab.as_deref())
            .bind(a.spawn_vfx.as_deref())
            .bind(a.icon.as_deref())
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(())
}
