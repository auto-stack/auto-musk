//! PLAN-092 T-03: auto-plan 四技能产品自带——musk 为真源，serve 为分发点。
//!
//! 产品内 agent（aaid）只扫 `<config>/autoos/skills/`（全局技能目录）；
//! 仓内 `.agents/skills/auto-plan-*` 是 ZCode 侧与产品的单一维护点。serve
//! 启动时把四技能幂等同步过去：目标存在且内容相同则跳过，不同则覆盖
//! （真源单一策略，PLAN-092 §10-3 默认）。源目录缺失仅 warn 不阻断 serve。

use std::path::{Path, PathBuf};

/// Vendored 技能名（仓内 `.agents/skills/` 真源，缺一即 warn）。
const BUILTIN_SKILLS: [&str; 4] = [
    "auto-plan-new",
    "auto-plan-work",
    "auto-plan-review",
    "auto-plan-merge",
];

/// 解析 vendored 技能源根：`MUSK_SKILLS_DIR` env → 运行 CWD `.agents/skills`
/// → 构建期仓库（CARGO_MANIFEST_DIR 回溯仓根；exe 异地运行时兜底）。
fn skills_source_root() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("MUSK_SKILLS_DIR") {
        let p = PathBuf::from(dir);
        if p.is_dir() {
            return Some(p);
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        let p = cwd.join(".agents").join("skills");
        if p.is_dir() {
            return Some(p);
        }
    }
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join(".agents")
        .join("skills")
        .canonicalize()
        .ok();
    match p {
        Some(p) if p.is_dir() => Some(p),
        _ => None,
    }
}

/// 递归比较拷贝：内容相同跳过，不同/缺失覆盖。返回实际写入文件数。
fn copy_tree_diff(src: &Path, dst: &Path) -> std::io::Result<usize> {
    let mut written = 0usize;
    if !src.is_dir() {
        return Ok(0);
    }
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let s = entry.path();
        let d = dst.join(entry.file_name());
        if file_type.is_dir() {
            written += copy_tree_diff(&s, &d)?;
        } else {
            let changed = match std::fs::read(&d) {
                Ok(cur) => cur != std::fs::read(&s)?,
                Err(_) => true,
            };
            if changed {
                std::fs::copy(&s, &d)?;
                written += 1;
            }
        }
    }
    Ok(written)
}

/// PLAN-092 T-03: 用户级技能目录单源——`MUSK_CONFIG_DIR` 覆盖时 agent 技能
/// 扫描目录与 serve 分发目标保持一致（隔离部署下技能可见），缺省
/// `~/.config/autoos/skills`。build_agent_from_mode 与本模块共用。
pub fn autoos_skills_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("MUSK_CONFIG_DIR") {
        return Some(PathBuf::from(dir).join("skills"));
    }
    dirs::home_dir().map(|h| h.join(".config/autoos/skills"))
}

/// serve 启动序列调用（server::serve 内、WorkspaceRegistry 装配处）。
pub fn sync_builtin_skills(config_dir: &Path) {
    let src_root = match skills_source_root() {
        Some(p) => p,
        None => {
            tracing::warn!("builtin skills source not found (.agents/skills) — sync skipped");
            return;
        }
    };
    let dst_root = config_dir.join("skills");
    if let Err(e) = std::fs::create_dir_all(&dst_root) {
        tracing::warn!("skills dir create failed ({}): {e}", dst_root.display());
        return;
    }
    for name in BUILTIN_SKILLS {
        let src = src_root.join(name);
        if !src.is_dir() {
            tracing::warn!("builtin skill missing in source: {}", src.display());
            continue;
        }
        let dst = dst_root.join(name);
        match copy_tree_diff(&src, &dst) {
            Ok(n) if n > 0 => {
                tracing::info!("builtin skill synced: {name} ({n} files) -> {}", dst.display())
            }
            Ok(_) => {}
            Err(e) => tracing::warn!("builtin skill sync failed ({name}): {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_is_idempotent_and_overwrites_changed() {
        let tmp = std::env::temp_dir().join(format!("musk-bskills-{}", std::process::id()));
        let src = tmp.join("src").join("auto-plan-new");
        let dst_root = tmp.join("cfg").join("skills");
        std::fs::create_dir_all(src.join("locales")).unwrap();
        std::fs::write(src.join("SKILL.md"), "v1").unwrap();
        std::fs::write(src.join("locales").join("zh-CN.json"), "{}").unwrap();

        // 首次同步：2 文件落盘。
        assert_eq!(copy_tree_diff(&src, &dst_root.join("auto-plan-new")).unwrap(), 2);
        // 幂等：再跑零写入。
        assert_eq!(copy_tree_diff(&src, &dst_root.join("auto-plan-new")).unwrap(), 0);
        // 真源更新：覆盖 1 文件。
        std::fs::write(src.join("SKILL.md"), "v2").unwrap();
        assert_eq!(copy_tree_diff(&src, &dst_root.join("auto-plan-new")).unwrap(), 1);
        assert_eq!(std::fs::read_to_string(dst_root.join("auto-plan-new").join("SKILL.md")).unwrap(), "v2");
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
