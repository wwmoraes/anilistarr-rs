use serde::Serialize;

pub type CustomList = Vec<CustomEntry>;

#[derive(Debug, Serialize)]
pub struct CustomEntry {
	pub tvdb_id: u64,
}
