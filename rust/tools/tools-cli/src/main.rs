//! 将 assets/office_sell.xlsx 解析为 JSON 并输出到 godot/assets/data/
//! 运行方式：在项目根目录执行 `cargo run -p tools`，或进入 rust 目录后 `cargo run -p tools`

use calamine::{open_workbook, Data, DataType, Range, Reader, Xlsx};
use model::office_sell::OfficeSellItem;
use serde_json;
use std::fs;
use std::path::{Path, PathBuf};

/// Excel 列顺序：id, name, sell, stack, in_shop（第 0 行为表头则跳过）
const COL_ID: usize = 0;
const COL_NAME: usize = 1;
const COL_SELL: usize = 2;
const COL_STACK: usize = 3;
const COL_IN_SHOP: usize = 4;

fn cell_as_i32(cell: &Data) -> Option<i32> {
    cell.as_i64().and_then(|n| i32::try_from(n).ok())
}

fn cell_as_i64(cell: &Data) -> Option<i64> {
    cell.as_i64().or_else(|| cell.as_f64().map(|f| f as i64))
}

fn cell_as_bool(cell: &Data) -> Option<bool> {
    cell.get_bool()
        .or_else(|| cell.as_i64().map(|n| n != 0))
        .or_else(|| {
            cell.get_string().map(|s| {
            let s = s.to_lowercase();
                if s == "true" || s == "是" || s == "1" || s == "y" || s == "yes" {
                    Some(true)
                } else if s == "false" || s == "否" || s == "0" || s == "n" || s == "no" {
                    Some(false)
            } else {
                None
            }
        })
    }.flatten())
}

fn is_header_row(row: &[Data]) -> bool {
    row.get(COL_ID)
        .and_then(|c| c.get_string())
        .map(|s| s.to_lowercase().trim() == "id")
        .unwrap_or(false)
}

fn parse_row(row: &[Data]) -> Option<OfficeSellItem> {
    let id = cell_as_i32(row.get(COL_ID)?)?;
    let name = row
        .get(COL_NAME)
        .and_then(|c| c.get_string())
        .map(String::from)
        .unwrap_or_default();
    let sell = cell_as_i64(row.get(COL_SELL)?).unwrap_or(0);
    let stack = cell_as_bool(row.get(COL_STACK)?).unwrap_or(false);
    let in_shop = cell_as_bool(row.get(COL_IN_SHOP)?).unwrap_or(false);

    Some(OfficeSellItem {
        id,
        name,
        sell,
        stack,
        in_shop,
    })
}

/// 解析单个工作表，返回该 sheet 内的所有有效行（自动识别并跳过表头行）
fn parse_sheet(range: &Range<Data>) -> Vec<OfficeSellItem> {
    let mut items = Vec::new();
    let mut rows = range.rows();
    let first_row = rows.next();
    let skip_header = first_row.map(|r| is_header_row(r)).unwrap_or(false);

    if let Some(row) = first_row {
        if !skip_header {
            if let Some(item) = parse_row(row) {
                items.push(item);
            }
        }
    }

    for row in rows {
        if let Some(item) = parse_row(row) {
            items.push(item);
        }
    }
    items
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 支持从项目根或 rust 目录运行
    let project_root: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|p| p.join("assets").join("office_sell.xlsx").exists())
        .map(|p| p.to_path_buf())
        .or_else(|| {
            let cwd = std::env::current_dir().ok()?;
            if cwd.join("assets").join("office_sell.xlsx").exists() {
                Some(cwd)
            } else {
                cwd.ancestors()
                    .find(|p| p.join("assets").join("office_sell.xlsx").exists())
                    .map(|p| p.to_path_buf())
            }
        })
        .unwrap_or_else(|| PathBuf::from("."));

    let xlsx_path = project_root.join("assets").join("office_sell.xlsx");
    let out_dir = project_root.join("godot").join("assets").join("data");
    let out_path = out_dir.join("office_sell.json");

    if !xlsx_path.exists() {
        return Err(format!("Excel 文件不存在: {}", xlsx_path.display()).into());
    }

    let mut workbook: Xlsx<_> = open_workbook(&xlsx_path)?;
    let sheet_names = workbook.sheet_names().to_owned();

    let mut list: Vec<OfficeSellItem> = Vec::new();
    for sheet_name in &sheet_names {
        let range = workbook
            .worksheet_range(sheet_name)
            .map_err(|e| format!("读取工作表 '{}' 失败: {}", sheet_name, e))?;
        let sheet_items = parse_sheet(&range);
        println!("  {}: {} 条", sheet_name, sheet_items.len());
        list.extend(sheet_items);
    }

    fs::create_dir_all(&out_dir)?;
    let json = serde_json::to_string_pretty(&list)?;
    fs::write(&out_path, json)?;

    println!(
        "已解析 {} 条记录，已写入: {}",
        list.len(),
        out_path.display()
    );
    Ok(())
}
