//! PLAN-090 T-03: 词汇表注入（stdlib 7 分类 53 widget 名单 + tier 统计 + safe subset）
//!
//! 优先运行时从 auto-lang 探测 stdlib 目录；环境不可达时由内嵌快照兜底（随版本演进）。

use std::path::PathBuf;

/// 内嵌快照兜底（数据源：auto-lang stdlib/aura/widgets/，53 件 7 分类）
pub const STDLIB_WIDGETS_SNAPSHOT: &[(&str, &[&str])] = &[
    (
        "data",
        &["Calendar", "DataTable", "Grid", "GridItem", "List", "ListItem", "Table"],
    ),
    (
        "display",
        &[
            "Avatar", "Badge", "Icon", "Image", "ImageSurface", "Separator", "Skeleton",
            "Swiper", "Text",
        ],
    ),
    ("feedback", &["Alert", "Progress", "Sonner", "Toast"]),
    (
        "form",
        &[
            "Button", "Checkbox", "Form", "Input", "RadioGroup", "Select", "Slider", "Switch",
            "Textarea",
        ],
    ),
    (
        "layout",
        &[
            "Accordion", "AspectRatio", "Card", "Center", "Col", "Collapsible", "Row",
            "ScrollArea",
        ],
    ),
    (
        "navigation",
        &[
            "Breadcrumb", "DropdownMenu", "MenuBar", "NavigationMenu", "NavLink",
            "Pagination", "Sidebar", "Tabs",
        ],
    ),
    (
        "overlay",
        &[
            "AlertDialog", "ContextMenu", "Dialog", "Drawer", "HoverCard", "Popover",
            "Sheet", "Tooltip",
        ],
    ),
];

/// 解析 auto-lang 仓库根目录
pub fn resolve_auto_lang_root() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("AUTO_LANG_ROOT") {
        if !p.trim().is_empty() {
            let path = PathBuf::from(p);
            if path.exists() {
                return Some(path);
            }
        }
    }
    let sibling = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../auto-lang");
    if sibling.exists() {
        return Some(sibling);
    }
    let main_checkout = PathBuf::from("D:/autostack/auto-lang");
    if main_checkout.exists() {
        return Some(main_checkout);
    }
    None
}

/// 探测运行时 stdlib 词汇，若不可达或异常则返回内嵌快照
pub fn load_widget_vocabulary() -> (Vec<(String, Vec<String>)>, bool) {
    if let Some(root) = resolve_auto_lang_root() {
        let widgets_dir = root.join("stdlib/aura/widgets");
        if widgets_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&widgets_dir) {
                let mut categories = Vec::new();
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let cat_name = entry.file_name().to_string_lossy().to_string();
                        let mut items = Vec::new();
                        if let Ok(files) = std::fs::read_dir(&path) {
                            for f in files.flatten() {
                                let fpath = f.path();
                                if fpath.is_file()
                                    && fpath.extension().and_then(|s| s.to_str()) == Some("at")
                                {
                                    let stem = fpath
                                        .file_stem()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .to_string();
                                    if stem != "mod" {
                                        items.push(stem);
                                    }
                                }
                            }
                        }
                        if !items.is_empty() {
                            items.sort();
                            categories.push((cat_name, items));
                        }
                    }
                }
                if !categories.is_empty() {
                    categories.sort_by(|a, b| a.0.cmp(&b.0));
                    return (categories, false);
                }
            }
        }
    }

    // 兜底降级快照
    let snapshot: Vec<(String, Vec<String>)> = STDLIB_WIDGETS_SNAPSHOT
        .iter()
        .map(|(cat, list)| {
            (
                cat.to_string(),
                list.iter().map(|s| s.to_string()).collect(),
            )
        })
        .collect();
    (snapshot, true)
}

/// 生成注入系统上下文的精简词汇表摘要（≤1.2KB）
pub fn get_vocabulary_summary() -> String {
    let (categories, degraded) = load_widget_vocabulary();
    let total_widgets: usize = categories.iter().map(|(_, items)| items.len()).sum();

    let mut out = format!(
        "### Widget Vocabulary (stdlib {} categories, {} widgets; 114 builtin widgets{})\n",
        categories.len(),
        total_widgets,
        if degraded { " [embedded-snapshot]" } else { "" }
    );

    for (cat, items) in &categories {
        out.push_str(&format!("- {}: {}\n", cat, items.join(", ")));
    }

    out.push_str(
        "Safe subset (proven/fast): col, row, text, button, input, icon, image, span, checkbox, select, badge, separator\n"
    );

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vocabulary_snapshot_count() {
        assert_eq!(STDLIB_WIDGETS_SNAPSHOT.len(), 7, "must have 7 categories");
        let total: usize = STDLIB_WIDGETS_SNAPSHOT.iter().map(|(_, items)| items.len()).sum();
        assert_eq!(total, 53, "must have exactly 53 widgets in snapshot");
    }

    #[test]
    fn test_load_widget_vocabulary_contains_all_categories() {
        let (categories, _degraded) = load_widget_vocabulary();
        assert_eq!(categories.len(), 7);
        let names: Vec<String> = categories.iter().map(|(c, _)| c.clone()).collect();
        assert!(names.contains(&"data".to_string()));
        assert!(names.contains(&"display".to_string()));
        assert!(names.contains(&"feedback".to_string()));
        assert!(names.contains(&"form".to_string()));
        assert!(names.contains(&"layout".to_string()));
        assert!(names.contains(&"navigation".to_string()));
        assert!(names.contains(&"overlay".to_string()));
    }

    #[test]
    fn test_vocabulary_summary_budget() {
        let summary = get_vocabulary_summary();
        assert!(summary.len() > 200);
        assert!(summary.len() < 1800, "vocabulary summary must be under 1.8KB, actual: {}", summary.len());
        assert!(summary.contains("Safe subset"));
        assert!(summary.contains("Button"));
        assert!(summary.contains("Calendar"));
    }
}
