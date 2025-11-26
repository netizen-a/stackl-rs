// Copyright (c) 2024-2025 Jonathan Thomason

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
			DataLayout::Array(inner) => self.type_array(inner),
			other => todo!("[resolve_type]: `{other:?}`"),
		}
	}
	pub(super) fn type_bool(&mut self) -> u32 {
		todo!("type_bool")
	}
	pub(super) fn type_void(&mut self) -> u32 {
		if let Some(id) = self.type_map.get(&DataLayout::Void) {
			*id
		} else {
			let id = self.builder.type_void();
			if let Some(value) = self.type_map.insert(DataLayout::Void, id) {
				let info = Diagnostic::info(
					DiagKind::Trace(format!("type_void id {id} already exists")),
					None,
				);
				if self.is_traced {
					self.diag_engine.push(info);
				}
			}
			id
		}
	}
	pub(super) fn type_int(&mut self, layout: &IntegerLayout) -> u32 {
		if let Some(id) = self.type_map.get(&DataLayout::Integer(layout.clone())) {
			*id
		} else {
			let id = self.builder.type_int(layout.width, layout.is_signed);
			if let Some(value) = self.type_map.insert(DataLayout::Void, id) {
				let info = Diagnostic::info(
					DiagKind::Trace(format!("type_void id {id} already exists")),
					None,
				);
				if self.is_traced {
					self.diag_engine.push(info);
				}
			}
			id
		}
	}
	pub(super) fn type_ptr(&mut self, layout: &PtrLayout) -> u32 {
		if let Some(id) = self.type_map.get(&DataLayout::Pointer(layout.clone())) {
			*id
		} else {
			let inner_id = self.resolve_type(&layout.0);
			let id = self.builder.type_pointer(inner_id);
			if let Some(value) = self.type_map.insert(DataLayout::Void, id) {
				let info = Diagnostic::info(
					DiagKind::Trace(format!("type_ptr id {id} already exists")),
					None,
				);
				if self.is_traced {
					self.diag_engine.push(info);
				}
			}
			id
		}
	}
	pub(super) fn type_array(&mut self, layout: &ArrayLayout) -> u32 {
		if let Some(id) = self.type_map.get(&DataLayout::Array(layout.clone())) {
			*id
		} else {
			let inner_id = self.resolve_type(&layout.component);
			let id = self.builder.type_array(inner_id, layout.length);
			if let Some(value) = self.type_map.insert(DataLayout::Void, id) {
				let info = Diagnostic::info(
					DiagKind::Trace(format!("type_array id {id} already exists")),
					None,
				);
				if self.is_traced {
					self.diag_engine.push(info);
				}
			}
			id
		}
	}
}
