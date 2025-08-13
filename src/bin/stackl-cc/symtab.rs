use std::{collections::HashMap, hash::Hash};

#[derive(Debug)]
pub enum SymbolTableError {
	InvalidScope,
	AlreadyExists,
	// DoesNotExist,
}

#[derive(Debug)]
pub struct SymbolTable<K, V> {
	table: Vec<HashMap<K, V>>,
}

impl<K: Eq + Hash, V> SymbolTable<K, V> {
	pub fn new() -> Self {
		Self {
			table: vec![HashMap::new()],
		}
	}
	pub fn lookup(&self, name: &K) -> Option<&V> {
		for current_scope in self.table.iter().rev() {
			if let Some(val) = current_scope.get(name) {
				return Some(val);
			}
		}
		None
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
	) -> Result<(), SymbolTableError> {
		let Some(table) = self.table.last_mut() else {
			return Err(SymbolTableError::InvalidScope);
		};
		match table.insert(key.into(), value.into()) {
			Some(_) => Err(SymbolTableError::AlreadyExists),
			None => Ok(()),
		}
	}
	// TODO: replace with insert_or_update
	// pub fn remove(&mut self, name: &K) -> Result<(), SymbolTableError> {
	//     let Some(table) = self.table.last_mut() else {
	//         return Err(SymbolTableError::InvalidScope);
	//     };
	//     match table.remove(name) {
	//         Some(_) => Ok(()),
	//         None => Err(SymbolTableError::DoesNotExist),
	//     }
	// }
}
