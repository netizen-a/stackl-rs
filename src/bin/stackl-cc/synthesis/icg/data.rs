// Copyright (c) 2024-2026 Jonathan A. Thomason

use std::collections::HashSet;

use crate::synthesis::icg::*;

impl SSACodeGen<'_> {
	pub(super) fn parse_types(&mut self, type_set: HashSet<DataLayout>) {
		for layout in type_set.iter() {
			self.resolve_type(layout);
		}
	}
	pub(super) fn resolve_type(&mut self, layout: &DataLayout) -> u32 {
		match layout {
			DataLayout::Bool => self.type_bool(),
			DataLayout::Void => self.type_void(),
			DataLayout::Integer(inner) => self.type_int(inner),
			DataLayout::Pointer(inner) => self.type_ptr(inner),
			DataLayout::Array(inner) => self.type_array(inner),
			DataLayout::Function(inner) => self.type_function(inner),
			DataLayout::Struct(inner) => self.type_struct(inner),
			other => todo!("[resolve_type]: `{other:?}`"),
		}
	}
	fn type_bool(&mut self) -> u32 {
		if let Some(id) = self.type_map.get(&DataLayout::Bool) {
			*id
		} else {
			let id = self.builder.type_bool();
			if let Some(value) = self.type_map.insert(DataLayout::Bool, id) {
				let info =
					Diagnostic::info(DiagKind::Trace(format!("SSA id {id} already exists")), None);
				if self.is_traced {
					self.diag_engine.push(info);
				}
			}
			id
		}
	}
	fn type_void(&mut self) -> u32 {
		if let Some(id) = self.type_map.get(&DataLayout::Void) {
			*id
		} else {
			let id = self.builder.type_void();
			if let Some(value) = self.type_map.insert(DataLayout::Void, id) {
				let info =
					Diagnostic::info(DiagKind::Trace(format!("SSA id {id} already exists")), None);
				if self.is_traced {
					self.diag_engine.push(info);
				}
			}
			id
		}
	}
	fn type_int(&mut self, layout: &IntegerLayout) -> u32 {
		let key = DataLayout::Integer(layout.clone());
		if let Some(id) = self.type_map.get(&key) {
			*id
		} else {
			let id = self.builder.type_int(layout.width, layout.is_signed);
			if let Some(value) = self.type_map.insert(key, id) {
				let info =
					Diagnostic::info(DiagKind::Trace(format!("SSA id {id} already exists")), None);
				if self.is_traced {
					self.diag_engine.push(info);
				}
			}
			id
		}
	}
	fn type_ptr(&mut self, layout: &PtrLayout) -> u32 {
		let key = DataLayout::Pointer(layout.clone());
		if let Some(id) = self.type_map.get(&key) {
			*id
		} else {
			let inner_type_id = self.resolve_type(&layout.0);
			let id = self.builder.type_pointer(inner_type_id);
			if let Some(value) = self.type_map.insert(key, id) {
				let info =
					Diagnostic::info(DiagKind::Trace(format!("SSA id {id} already exists")), None);
				if self.is_traced {
					self.diag_engine.push(info);
				}
			}
			id
		}
	}
	fn type_array(&mut self, layout: &ArrayLayout) -> u32 {
		let key = DataLayout::Array(layout.clone());
		if let Some(id) = self.type_map.get(&key) {
			*id
		} else {
			let inner_id = self.resolve_type(&layout.component);
			let id = self.builder.type_array(inner_id, layout.length);
			if let Some(value) = self.type_map.insert(key, id) {
				let info =
					Diagnostic::info(DiagKind::Trace(format!("SSA id {id} already exists")), None);
				if self.is_traced {
					self.diag_engine.push(info);
				}
			}
			id
		}
	}

	fn type_function(&mut self, layout: &FunctionLayout) -> u32 {
		let key = DataLayout::Function(layout.clone());
		if let Some(id) = self.type_map.get(&key) {
			*id
		} else {
			let ret_id = self.resolve_type(&layout.ret);
			let param_ids: Box<[u32]> = layout
				.params
				.iter()
				.map(|param| self.resolve_type(&param))
				.collect();
			let id: u32 = if layout.is_variadic {
				self.builder
					.type_variadic_function(ret_id, &param_ids)
					.unwrap()
			} else {
				self.builder.type_function(ret_id, &param_ids).unwrap()
			};
			if let Some(value) = self.type_map.insert(key, id) {
				let info =
					Diagnostic::info(DiagKind::Trace(format!("SSA id {id} already exists")), None);
				if self.is_traced {
					self.diag_engine.push(info);
				}
			}
			id
		}
	}

	fn type_struct(&mut self, layout: &StructLayout) -> u32 {
		let key = DataLayout::Struct(layout.clone());
		if let Some(id) = self.type_map.get(&key) {
			*id
		} else {
			let member_ids: Box<[u32]> = layout
				.0
				.iter()
				.map(|member| self.resolve_type(&member))
				.collect();
			let id = self.builder.type_struct(&member_ids);
			if let Some(value) = self.type_map.insert(key, id) {
				let info =
					Diagnostic::info(DiagKind::Trace(format!("SSA id {id} already exists")), None);
				if self.is_traced {
					self.diag_engine.push(info);
				}
			}
			id
		}
	}
}
