use std::{
	collections::{hash_map, HashMap},
	hash::Hash,
};

use crate::{analysis::syn, data_types::DataType, diagnostics::Span};

#[derive(Debug)]
pub enum SymbolTableError<V: Clone> {
	InvalidScope,
	AlreadyExists(V),
	// DoesNotExist,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Namespace {
	Label(String),
	Tag(String),
	Member { tag: String, member: String },
	Ordinary(String),
}

#[derive(Debug, Clone, Copy)]
pub enum Linkage {
	None,
	External,
	Internal,
}

#[derive(Debug, Clone)]
pub struct SymbolTableEntry {
	pub data_type: DataType,
	pub storage: syn::StorageClass,
	pub linkage: Linkage,
	pub span: Span,
}

#[derive(Debug)]
pub struct SymbolTable<K = Namespace, V = SymbolTableEntry> {
	table: Vec<HashMap<K, V>>,
}

impl<K: Eq + Hash, V: Clone> Default for SymbolTable<K, V> {
	fn default() -> Self {
		Self::new()
	}
}

impl<K: Eq + Hash, V: Clone> SymbolTable<K, V> {
	pub fn new() -> Self {
		Self {
			table: vec![HashMap::new()],
		}
	}
	pub fn global_lookup(&self, name: &K) -> Option<&V> {
		for current_scope in self.table.iter().rev() {
			if let Some(val) = current_scope.get(name) {
				return Some(val);
			}
		}
		None
	}
	pub fn local_lookup(&self, name: &K) -> Option<&V> {
		self.table.last().and_then(|v| v.get(name))
	}
	pub fn increase_scope(&mut self) {
		self.table.push(HashMap::new())
	}
	pub fn decrease_scope(&mut self) -> bool {
		self.table.pop().is_some()
	}
	pub fn insert(
		&mut self,
		key: impl Into<K>,
		value: impl Into<V>,
	) -> Result<(), SymbolTableError<V>> {
		let key = key.into();
		let Some(table) = self.table.last_mut() else {
			return Err(SymbolTableError::InvalidScope);
		};
		match table.get(&key) {
			Some(value) => Err(SymbolTableError::AlreadyExists(value.clone())),
			None => {
				table.insert(key, value.into());
				Ok(())
			}
		}
	}
	pub fn iter_current_scope(&self) -> Option<hash_map::Iter<K, V>> {
		self.table.last().and_then(|map| Some(map.iter()))
	}
	pub fn scope_count(&self) -> usize {
		self.table.len()
	}
}
