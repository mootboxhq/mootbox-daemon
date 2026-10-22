use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use proc_macro2::{Group, TokenStream, TokenTree};
use schemars::schema::Schema;
use serde_json::Value;
use syn::visit_mut::{self, VisitMut};
use typify::{TypeSpace, TypeSpaceSettings};

const SCHEMA_DIR: &str = "contracts/runner/schema";
const INDEX_FILE: &str = "index.json";
const OUTPUT_FILE: &str = "src/contracts.rs";
const SCHEMA_EXTENSION: &str = ".json";
const CHECK_FLAG: &str = "--check";

type Outcome<T> = Result<T, Box<dyn Error>>;

fn main() -> Outcome<ExitCode> {
    let root = workspace_root()?;
    let output = root.join(OUTPUT_FILE);
    let generated = generated_contracts(&root)?;
    if std::env::args().any(|argument| argument == CHECK_FLAG) {
        return drift_status(&output, &generated);
    }
    fs::write(&output, generated).map_err(|error| format!("cannot write {}: {error}", output.display()))?;
    println!("wrote {}", output.display());
    Ok(ExitCode::SUCCESS)
}

fn drift_status(output: &Path, generated: &str) -> Outcome<ExitCode> {
    let committed = fs::read_to_string(output).map_err(|error| format!("cannot read {}: {error}", output.display()))?;
    if committed == generated {
        return Ok(ExitCode::SUCCESS);
    }
    eprintln!("{} is out of date; run cargo run -p gen-contracts", output.display());
    Ok(ExitCode::FAILURE)
}

fn generated_contracts(root: &Path) -> Outcome<String> {
    let schema_dir = root.join(SCHEMA_DIR);
    let mut type_space = TypeSpace::new(&TypeSpaceSettings::default());
    for file in schema_files(&schema_dir)? {
        let name = type_name(&file);
        let mut json = read_json(&schema_dir.join(&file))?;
        title_tagged_variant_objects(&name, &mut json);
        let schema: Schema =
            serde_json::from_value(json).map_err(|error| format!("{file} is not a JSON Schema: {error}"))?;
        type_space
            .add_type_with_name(&schema, Some(name))
            .map_err(|error| format!("typify rejected {file}: {error}"))?;
    }
    let tokens = without_doc_attributes(type_space.to_stream());
    let mut file: syn::File = syn::parse2(tokens).map_err(|error| format!("typify produced invalid Rust: {error}"))?;
    BoxedVariantPayloads.visit_file_mut(&mut file);
    file.attrs.push(syn::parse_quote!(#![allow(clippy::unwrap_used)]));
    formatted_with_rustfmt(root, &prettyplease::unparse(&file))
}

fn workspace_root() -> Outcome<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("gen-contracts must live inside the workspace")?;
    Ok(root.to_path_buf())
}

fn read_json(path: &Path) -> Outcome<Value> {
    let text = fs::read_to_string(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    Ok(serde_json::from_str(&text).map_err(|error| format!("{} is not JSON: {error}", path.display()))?)
}

fn schema_files(schema_dir: &Path) -> Outcome<BTreeSet<String>> {
    let index = read_json(&schema_dir.join(INDEX_FILE))?;
    let error_file = index["error"].as_str().ok_or("index.json has no error schema")?;
    let routes = index["routes"].as_array().ok_or("index.json has no routes")?;
    let route_files = routes.iter().flat_map(|route| {
        ["request", "response"]
            .into_iter()
            .filter_map(|key| route.get(key))
            .map(|file| {
                file.as_str()
                    .map(str::to_string)
                    .ok_or("a route names a schema that is not a string")
            })
    });
    let mut files = route_files.collect::<Result<BTreeSet<_>, _>>()?;
    files.insert(error_file.to_string());
    Ok(files)
}

fn type_name(file: &str) -> String {
    pascal_case(file.trim_end_matches(SCHEMA_EXTENSION))
}

fn pascal_case(words: &str) -> String {
    words.split(['-', '.', '_']).map(capitalized).collect()
}

fn title_tagged_variant_objects(type_name: &str, schema: &mut Value) {
    let Some(variants) = schema.get_mut("oneOf").and_then(Value::as_array_mut) else {
        return;
    };
    for variant in variants {
        let Some(properties) = variant.get_mut("properties").and_then(Value::as_object_mut) else {
            continue;
        };
        let Some(tag) = properties.values().find_map(|property| property.get("const")?.as_str()) else {
            continue;
        };
        let variant_name = format!("{type_name}{}", pascal_case(tag));
        for (property_name, property) in properties.iter_mut() {
            let is_untitled_object = property["type"] == "object" && property.get("title").is_none();
            if is_untitled_object {
                property["title"] = Value::from(format!("{variant_name}{}", pascal_case(property_name)));
            }
        }
    }
}

fn capitalized(word: &str) -> String {
    let mut characters = word.chars();
    characters
        .next()
        .map(|first| first.to_ascii_uppercase().to_string() + characters.as_str())
        .unwrap_or_default()
}

fn without_doc_attributes(tokens: TokenStream) -> TokenStream {
    let mut kept = Vec::new();
    let mut trees = tokens.into_iter().peekable();
    while let Some(tree) = trees.next() {
        if is_attribute_start(&tree) && trees.peek().is_some_and(is_doc_attribute_body) {
            trees.next();
            continue;
        }
        kept.push(without_doc_attributes_in_group(tree));
    }
    kept.into_iter().collect()
}

fn without_doc_attributes_in_group(tree: TokenTree) -> TokenTree {
    let TokenTree::Group(group) = tree else {
        return tree;
    };
    let mut cleaned = Group::new(group.delimiter(), without_doc_attributes(group.stream()));
    cleaned.set_span(group.span());
    TokenTree::Group(cleaned)
}

fn is_attribute_start(tree: &TokenTree) -> bool {
    matches!(tree, TokenTree::Punct(punct) if punct.as_char() == '#')
}

fn is_doc_attribute_body(tree: &TokenTree) -> bool {
    let TokenTree::Group(group) = tree else {
        return false;
    };
    let first = group.stream().into_iter().next();
    matches!(first, Some(TokenTree::Ident(ident)) if ident == "doc")
}

fn formatted_with_rustfmt(root: &Path, source: &str) -> Outcome<String> {
    let mut rustfmt = Command::new("rustfmt")
        .args(["--edition", "2024"])
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|error| format!("cannot run rustfmt: {error}"))?;
    rustfmt
        .stdin
        .take()
        .ok_or("rustfmt stdin is not piped")?
        .write_all(source.as_bytes())?;
    let output = rustfmt.wait_with_output()?;
    if !output.status.success() {
        return Err("rustfmt failed on the generated contracts".into());
    }
    Ok(String::from_utf8(output.stdout)?)
}

struct BoxedVariantPayloads;

impl VisitMut for BoxedVariantPayloads {
    fn visit_variant_mut(&mut self, variant: &mut syn::Variant) {
        for field in &mut variant.fields {
            if is_generated_type(&field.ty) {
                let payload = &field.ty;
                field.ty = syn::parse_quote!(::std::boxed::Box<#payload>);
            }
        }
        visit_mut::visit_variant_mut(self, variant);
    }
}

fn is_generated_type(field_type: &syn::Type) -> bool {
    let syn::Type::Path(path) = field_type else {
        return false;
    };
    let is_bare_name = path.qself.is_none() && path.path.leading_colon.is_none() && path.path.segments.len() == 1;
    is_bare_name && path.path.segments[0].ident.to_string().starts_with(char::is_uppercase)
}
