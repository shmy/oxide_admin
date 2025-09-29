use cruet::Inflector;
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionItem {
    label: String,
    key: String,
    #[serde(default)]
    value: Option<i32>,
    #[serde(default)]
    children: Vec<PermissionItem>,
}

fn main() {
    let content = fs::read("iam/permissions.yaml").unwrap();
    let tree: Vec<PermissionItem> = serde_yaml::from_slice(&content).unwrap();

    generate_rust(&tree);
    generate_javascript(&tree);

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=iam/permissions.yaml");
}

fn generate_rust(tree: &Vec<PermissionItem>) {
    let mut rs = String::new();

    let mut all_permissions = Vec::new();

    for t in tree {
        write_flat_rust(&mut rs, t, Vec::new(), &mut all_permissions);
    }

    // 生成 ALL_PERMISSIONS
    rs.push_str("\npub const ALL_PERMISSIONS: &[Permission] = &[\n");
    for code in all_permissions.iter() {
        writeln!(rs, "    {},", code).unwrap();
    }
    rs.push_str("];\n\n");

    // 生成 PERMISSION_TREE
    rs.push_str("pub const PERMISSION_TREE: &[PermissionTree] = &[\n");
    for t in tree {
        write_tree_to_rust(&mut rs, t, Vec::new());
    }
    rs.push_str("];\n");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("permissions.rs");

    fs::write(out_path, rs).unwrap();
}

// 平铺生成 Permission 常量
fn write_flat_rust(
    rs: &mut String,
    node: &PermissionItem,
    path: Vec<String>,
    all_permissions: &mut Vec<String>,
) {
    let mut new_path = path.clone();
    new_path.push(node.key.to_screaming_snake_case());

    let name = new_path.join("_");
    if let Some(value) = node.value {
        writeln!(
            rs,
            "pub const {}: Permission = Permission({});",
            name, value
        )
        .unwrap();
        all_permissions.push(name.clone());
    }

    for child in &node.children {
        write_flat_rust(rs, child, new_path.clone(), all_permissions);
    }
}

// 递归生成 PERMISSION_TREE
fn write_tree_to_rust(rs: &mut String, node: &PermissionItem, path: Vec<String>) {
    let indent = "    ".repeat(path.len() + 1);
    let key = node.key.to_screaming_snake_case();
    let mut new_path = path.clone();
    new_path.push(key.clone());

    let value_str = if node.value.is_some() {
        format!("Some({})", new_path.join("_"))
    } else {
        "None".to_string()
    };

    if !node.children.is_empty() {
        writeln!(rs, "{}PermissionTree {{", indent).unwrap();
        writeln!(rs, "{}    label: \"{}\",", indent, node.label).unwrap();
        writeln!(rs, "{}    value: {},", indent, value_str).unwrap();
        writeln!(rs, "{}    children: Some(&[", indent).unwrap();
        for child in &node.children {
            write_tree_to_rust(rs, child, new_path.clone());
        }
        writeln!(rs, "{}    ])", indent).unwrap();
        writeln!(rs, "{}}},", indent).unwrap();
    } else {
        writeln!(
            rs,
            "{}PermissionTree {{ label: \"{}\", value: {}, children: None }},",
            indent, node.label, value_str
        )
        .unwrap();
    }
}

// 生成 JS 对象
fn generate_javascript(tree: &Vec<PermissionItem>) {
    let mut js = String::new();
    js.push_str("export const PERMISSIONS = {\n");
    for t in tree {
        write_to_js(&mut js, t, Vec::new());
    }
    js.push_str("};\n");

    fs::write("../../frontend/src/lib/permissions.ts", js).unwrap();
}

fn write_to_js(js: &mut String, node: &PermissionItem, path: Vec<String>) {
    let indent = "  ".repeat(path.len() + 1);
    let key = node.key.to_screaming_snake_case();
    let mut new_path = path.clone();
    new_path.push(key.clone());

    if !node.children.is_empty() {
        writeln!(js, "{}{}: {{", indent, key).unwrap();
        for child in &node.children {
            write_to_js(js, child, new_path.clone());
        }
        writeln!(js, "{}}},", indent).unwrap();
    } else if let Some(value) = node.value {
        writeln!(js, "{}{}: {},", indent, key, value).unwrap();
    }
}
