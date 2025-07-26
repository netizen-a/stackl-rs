use crate::analysis::syn::*;

impl super::IntermediateCodeGen {
	pub(super) fn function_definition(&mut self, decl: &mut FunctionDefinition) {
		for ref mut specifier in decl.declaration_specifiers.iter_mut() {
			self.declaration_specifier(specifier);
		}
		self.declarator(&mut decl.declarator);
		for declaration in decl.declaration_list.iter_mut() {
			self.declaration(declaration);
		}
		self.compound_stmt(&mut decl.compound_stmt);
	}
	pub(super) fn declaration(&mut self, decl: &mut Declaration) {
		for ref mut spec in decl.declaration_specifiers.iter_mut() {
			self.declaration_specifier(spec);
		}
		for ref mut init_decl in decl.init_declarator_list.iter_mut() {
			self.init_declarator(init_decl);
		}
	}
	pub(super) fn declaration_specifier(&mut self, specifier: &mut DeclarationSpecifier) {
		use DeclarationSpecifier::*;
		match specifier {
			StorageClassSpecifier(spec) => self.storage_class_specifier(spec),
			TypeSpecifier(spec) => self.type_specifier(spec),
			TypeQualifier(spec) => self.type_qualifier(spec),
			FunctionSpecifier(spec) => self.function_specifier(spec),
		}
	}
	pub(super) fn init_declarator(&mut self, decl: &mut InitDeclarator) {
		self.declarator(&mut decl.declarator);
		if let Some(ref mut init) = decl.initializer {
			self.initializer(init);
		}
	}
	pub(super) fn storage_class_specifier(&mut self, _spec: &mut StorageClassSpecifier) {
		todo!("storage-class-specifier")
	}
	pub(super) fn type_specifier(&mut self, spec: &mut TypeSpecifier) {
		use TypeSpecifier::*;
		match spec {
			Void => (),
			Char => (),
			Short => (),
			Int => (),
			Long => (),
			Float => (),
			Double => (),
			Signed => (),
			Unsigned => (),
			Bool => (),
			StructOrUnionSpecifier(spec) => self.struct_or_union_specifier(spec),
			EnumSpecifier(spec) => self.enum_specifier(spec),
			TypedefName(_name) => todo!(),
		}
	}
	pub(super) fn enum_specifier(&mut self, _spec: &mut EnumSpecifier) {
		todo!("enum-specifier")
	}
	pub(super) fn enumerator(&mut self, enumerator: &mut Enumerator) {
		if let Some(ref mut expr) = enumerator.constant_expr {
			self.expr(expr);
		}
	}

	pub(super) fn struct_declaration(&mut self, decl: &mut StructDeclaration) {
		for ref mut spec in decl.specifier_qualifier_list.iter_mut() {
			self.specifier_qualifier(spec);
		}
		for ref mut struct_decl in decl.struct_declaration_list.iter_mut() {
			self.struct_declarator(struct_decl);
		}
	}

	pub(super) fn struct_declarator(&mut self, struct_decl: &mut StructDeclarator) {
		if let Some(ref mut decl) = struct_decl.declarator {
			self.declarator(decl)
		}
		if let Some(ref mut expr) = struct_decl.constant_expr {
			self.expr(expr);
		}
		// todo!("struct-declarator")
	}
	pub(super) fn struct_or_union_specifier(&mut self, _spec: &mut StructOrUnionSpecifier) {
		todo!("struct-or-union-specifier")
	}
	pub(super) fn initializer(&mut self, init: &mut Initializer) {
		use Initializer::*;
		match init {
			Expr(expr) => self.expr(expr),
			InitializerList(list) => self.initializer_list(list),
		}
	}
	pub(super) fn function_specifier(&mut self, _spec: &mut FunctionSpecifier) {
		todo!("function-specifier")
	}
	pub(super) fn declarator(&mut self, decl: &mut Declarator) {
		for ref mut ptr in decl.pointer.iter_mut() {
			self.pointer(ptr);
		}
		for ref mut direct_decl in decl.direct_declarator.iter_mut() {
			self.direct_declarator(direct_decl);
		}
	}
	pub(super) fn direct_declarator(&mut self, direct_decl: &mut DirectDeclarator) {
		use DirectDeclarator::*;
		match direct_decl {
			Identifier(_) => (),
			Declarator(_) => todo!("direct-declarator decl"),
			Array {
				type_qualifier_list,
				assignment_expr,
				has_static,
				has_ptr,
			} => todo!("direct-declarator array"),
			ParameterTypeList(type_list) => {
				for param in type_list.parameter_list.iter_mut() {
					self.parameter_declaration(param);
				}
			}
			IdentifierList(_ident_list) => (),
		}
	}
	pub(super) fn parameter_type_list(&mut self, list: &mut ParameterTypeList) {
		for param in list.parameter_list.iter_mut() {
			self.parameter_declaration(param);
		}
	}
	pub(super) fn pointer(&mut self, ptr: &mut Pointer) {
		for qual in ptr.type_qualifier_list.iter_mut() {
			self.type_qualifier(qual);
		}
	}
	pub(super) fn type_qualifier(&mut self, qual: &mut TypeQualifier) {
		use TypeQualifier::*;
		match qual {
			Const => (),
			Restrict => (),
			Volatile => (),
		}
	}
	pub(super) fn parameter_declaration(&mut self, param: &mut ParameterDeclaration) {
		for ref mut specifier in param.declaration_specifiers.iter_mut() {
			self.declaration_specifier(specifier);
		}
		self.parameter_declarator(&mut param.parameter_declarator);
	}
	pub(super) fn parameter_declarator(&mut self, param_decl: &mut ParameterDeclarator) {
		use ParameterDeclarator::*;
		match param_decl {
			Declarator(decl) => self.declarator(decl),
			AbstractDeclarator(decl) => {
				if let Some(decl) = decl {
					self.abstract_declarator(decl)
				}
			}
		}
	}
	pub(super) fn abstract_declarator(&mut self, decl: &mut AbstractDeclarator) {
		use AbstractDeclarator::*;
		match decl {
			Pointer(ptr) => self.pointer(ptr),
			DirectAbstractDeclarator {
				pointer,
				direct_abstract_declarator,
			} => {
				self.pointer(pointer);
				for ref mut declarator in direct_abstract_declarator {
					self.direct_abstract_declarator(declarator);
				}
			}
		}
	}
	pub(super) fn direct_abstract_declarator(&mut self, decl: &mut DirectAbstractDeclarator) {
		use DirectAbstractDeclarator::*;
		match decl {
			AbstractDeclarator(abstract_decl) => self.abstract_declarator(abstract_decl),
			Array {
				direct_abstract_declarator,
				assignment_expr,
				has_static: _,
			} => {
				for qual in direct_abstract_declarator {
					self.type_qualifier(qual);
				}
				if let Some(expr) = assignment_expr {
					self.expr(expr);
				}
			}
			ArrayPointer => todo!("direct-abstract-declarator [ * ]"),
			ParameterTypeList(_) => todo!("parameter-type-list"),
		}
	}
	pub(super) fn specifier_qualifier(&mut self, spec: &mut SpecifierQualifier) {
		use SpecifierQualifier::*;
		match spec {
			TypeSpecifier(ty) => self.type_specifier(ty),
			TypeQualifier(ty) => self.type_qualifier(ty),
		}
	}
	pub(super) fn initializer_list(&mut self, list: &mut InitializerList) {
		for (desig, ref mut init) in list.0.iter_mut() {
			if let Some(ref mut desig) = desig {
				self.designation(desig);
			}
			self.initializer(init);
		}
	}
	pub(super) fn designation(&mut self, desig: &mut Designation) {
		for ref mut desig in desig.0.iter_mut() {
			self.designator(desig)
		}
	}
	pub(super) fn designator(&mut self, desig: &mut Designator) {
		use Designator::*;
		match desig {
			ConstantExpr(expr) => self.expr(expr),
			Dot(_) => (),
		}
	}
}
