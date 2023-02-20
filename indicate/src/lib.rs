#![deny(unsafe_code)]
#![feature(iter_collect_into)]
use std::{
    cell::RefCell, collections::BTreeMap, fs, path::Path, rc::Rc, sync::Arc,
};

use adapter::IndicateAdapter;
use cargo_metadata::{Metadata, MetadataCommand};
use lazy_static::lazy_static;
use serde::Deserialize;
use trustfall::{
    execute_query as trustfall_execute_query, FieldValue, Schema,
    TransparentValue,
};

mod adapter;
mod vertex;

const RAW_SCHEMA: &str = include_str!("schema.trustfall.graphql");

lazy_static! {
    static ref SCHEMA: Schema =
        Schema::parse(RAW_SCHEMA).expect("Could not parse schema!");
}

/// Type representing a thread-safe JSON object, like
/// ```json
/// {
///     "name": "hello",
///     "value": true,
/// }
/// ```
type ObjectMap = BTreeMap<Arc<str>, FieldValue>;

#[derive(Debug, Clone, Deserialize)]
struct Query<'a> {
    pub query: &'a str,
    pub args: ObjectMap,
}

/// Transform a result from [`execute_query`] to one where the fields can easily be
/// serialized to JSON using [`TransparentValue`].
pub fn transparent_results(
    res: Vec<BTreeMap<Arc<str>, FieldValue>>,
) -> Vec<BTreeMap<Arc<str>, TransparentValue>> {
    res.into_iter()
        .map(|entry| entry.into_iter().map(|(k, v)| (k, v.into())).collect())
        .collect()
}

/// Executes a Trustfall query at a defined path, using the schema
/// provided by `indicate`.
pub fn execute_query(
    query_path: &Path,
    metadata_path: &Path,
) -> Vec<BTreeMap<Arc<str>, FieldValue>> {
    let raw_query = fs::read_to_string(query_path)
        .expect("Could not read query at {path}!");

    let full_query = ron::from_str::<Query>(&raw_query)
        .expect("Could not deserialize query!");

    let metadata = extract_metadata_from_path(metadata_path);
    let adapter = Rc::new(RefCell::new(IndicateAdapter::new(&metadata)));
    let res = match trustfall_execute_query(
        &SCHEMA,
        adapter,
        full_query.query,
        full_query.args,
    ) {
        Err(e) => panic!("Could not execute query due to error: {:#?}", e),
        Ok(res) => res.collect(),
    };
    res
}

/// Extracts metadata from a `Cargo.toml` file by its direct path
pub fn extract_metadata_from_path(path: &Path) -> Metadata {
    MetadataCommand::new()
        .manifest_path(path)
        .exec()
        .unwrap_or_else(|_| {
            panic!("Could not extract metadata from path {:?}", path)
        })
}

#[cfg(test)]
mod test {
    // use lazy_static::lazy_static;
    use std::{fs, path::Path};
    use test_case::test_case;

    use crate::{execute_query, transparent_results};

    #[test_case("direct_dependencies", "direct_dependencies" ; "direct dependencies as listed in Cargo.toml")]
    #[test_case("direct_dependencies", "no_deps_all_fields" ; "retrieving all fields of root package, but not dependencies")]
    #[test_case("direct_dependencies", "dependency_package_info" ; "information about root package direct dependencies")]
    #[test_case("direct_dependencies", "recursive_dependency" ; "retrieve recursive dependency information")]
    fn query_tests(fake_crate: &str, query_name: &str) {
        let raw_cargo_toml_path =
            format!("test_data/fake_crates/{fake_crate}/Cargo.toml");
        let cargo_toml_path = Path::new(&raw_cargo_toml_path);

        let raw_query_path = format!("test_data/queries/{query_name}.in.ron");
        let query_path = Path::new(&raw_query_path);

        let raw_expected_result_path =
            format!("test_data/queries/{query_name}.expected.json");
        let expected_result_name = Path::new(&raw_expected_result_path);

        // We use `TransparentValue for neater JSON serialization
        let res =
            transparent_results(execute_query(query_path, cargo_toml_path));
        let res_json_string = serde_json::to_string_pretty(&res)
            .expect("Could not convert result to string");

        let expected_result_string = fs::read_to_string(expected_result_name)
            .unwrap_or_else(|_| {
                panic!(
                    "Could not read expected file '{}'",
                    expected_result_name.to_string_lossy()
                );
            });

        assert_eq!(
            res_json_string.trim(),
            expected_result_string.trim(),
            "\nfailing query result:\n{}\n but expected:\n{}\n",
            res_json_string,
            expected_result_string
        );
    }
}
