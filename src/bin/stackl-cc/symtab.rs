// Copyright (c) 2024-2026 Jonathan A. Thomason

use std::{
	collections::{
		HashMap,
		hash_map,
	},
	hash::Hash,
};

use crate::{
	analysis::syn,
	data_type::{
		DataType,
		TypeKind,
	},
	diagnostics::{
		Span,
		ToSpan,
	},
};

#[derive(Debug)]
pub enum SymbolTableError<V: Clone> {
	InvalidScope,
	AlreadyExists(V),
}

#[derive(Debug, Clone, Copy)]
pub enum Linkage {
	External,
	Internal,
}

#[derive(Debug, Clone, Copy)]
pub enum StorageClass {
	Automatic,
	Static,
	Typename,
	Constant,
	Register,
}

impl TryFrom<StorageClass> for stackl::ssa::data::StorageClass {
	type Error = ();
	fn try_from(value: StorageClass) -> Result<Self, Self::Error> {
		match value {
			StorageClass::Automatic => Ok(Self::Automatic),
			StorageClass::Register => Ok(Self::Automatic),
			StorageClass::Static => Ok(Self::Static),
			StorageClass::Constant | StorageClass::Typename | StorageClass::Constant => Err(()),
		}
	}
}

#[derive(Debug, Clone)]
pub struct SymbolTableEntry {
	pub data_type: DataType,
	pub storage: StorageClass,
	pub linkage: Linkage,
	pub span: Span,
	/// This distinguishes between function definition, or a declaration (which may be a prototype).
	/// If function definition then false, otherwise true.
	pub is_decl: bool,
}

impl SymbolTableEntry {
	/// checks if type is a compile-time constant
	pub fn is_constant(&self) -> bool {
		matches!(self.storage, StorageClass::Constant)
	}
}

impl ToSpan for SymbolTableEntry {
	fn to_span(&self) -> Span {
		self.span.clone()
	}
}

#[derive(Debug)]
pub struct SymbolTable<K = String, V = SymbolTableEntry> {
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
	// insert new entry, otherwise return existing
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
	// insert new entry, otherwise replace existing entry
	pub fn update(
		&mut self,
		key: impl Into<K>,
		value: impl Into<V>,
	) -> Result<(), SymbolTableError<V>> {
		let key = key.into();
		let Some(table) = self.table.last_mut() else {
			return Err(SymbolTableError::InvalidScope);
		};
		table.insert(key, value.into());
		Ok(())
	}
	pub fn iter_current_scope(&self) -> Option<hash_map::Iter<K, V>> {
		self.table.last().and_then(|map| Some(map.iter()))
	}
	pub fn scope_count(&self) -> usize {
		self.table.len()
	}
}
