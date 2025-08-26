use crate::analysis::syn::*;
use crate::analysis::tok;
use crate::diagnostics as diag;

impl super::SemanticParser<'_> {
	pub(super) fn function_definition(&mut self, decl: &mut FunctionDefinition) {
		println!("DEBUG function_definition: {:?}", decl.specifiers);
		self.declarator(&mut decl.declarator);
		for declaration in decl.declaration_list.iter_mut() {
			self.declaration(declaration);
		}
		self.compound_stmt(&mut decl.compound_stmt);
	}
	pub(super) fn declaration(&mut self, decl: &mut Declaration) {
		if decl.specifiers.storage_classes.len() > 1 {
			let storage_class = decl.specifiers.storage_classes.first().unwrap();
			let diag = diag::Diagnostic::error(
				diag::DiagKind::MultStorageClasses,
				storage_class.span.clone(),
			);
			self.diagnostics.push_sem(diag);
		}
		println!("DEBUG declaration: {:?}", decl.specifiers);
		for ref mut init_decl in decl.init_declarator_list.iter_mut() {
			self.init_declarator(init_decl);
		}
	}
	fn init_declarator(&mut self, decl: &mut InitDeclarator) {
		self.declarator(&mut decl.declarator);
		if let Some(ref mut init) = decl.initializer {
			self.initializer(init);
		}
	}
	fn enum_specifier(&mut self, _spec: &mut EnumSpecifier) {
		todo!("enum-specifier")
	}
	fn enumerator(&mut self, enumerator: &mut Enumerator) {
		if let Some(ref mut expr) = enumerator.constant_expr {
			self.expr(expr);
		}
	}

	fn struct_declaration(&mut self, decl: &mut StructDeclaration) {
		for ref mut spec in decl.specifier_qualifier_list.iter_mut() {
			self.specifier_qualifier(spec);
		}
		for ref mut struct_decl in decl.struct_declaration_list.iter_mut() {
			self.struct_declarator(struct_decl);
		}
	}

	fn struct_declarator(&mut self, struct_decl: &mut StructDeclarator) {
		if let Some(ref mut decl) = struct_decl.declarator {
			self.declarator(decl)
		}
		if let Some(ref mut expr) = struct_decl.constant_expr {
			self.expr(expr);
		}
	}
	fn struct_or_union_specifier(&mut self, _spec: &mut StructOrUnionSpecifier) {
		todo!("struct-or-union-specifier")
	}
	fn initializer(&mut self, init: &mut Initializer) {
		use Initializer::*;
		match init {
			Expr(expr) => self.expr(expr),
			InitializerList(list) => self.initializer_list(list),
		}
	}
	fn function_specifier(&mut self, _spec: &mut FunctionSpecifier) {
		todo!("function-specifier")
	}
	fn declarator(&mut self, decl: &mut Declarator) {
		for ref mut ptr in decl.pointer.iter_mut() {
			self.pointer(ptr);
		}
		for ref mut direct_decl in decl.direct_declarator.iter_mut() {
			self.direct_declarator(direct_decl);
		}
	}
	fn direct_declarator(&mut self, direct_decl: &mut DirectDeclarator) {
		use DirectDeclarator::*;
		match direct_decl {
			Declarator(_decl) => eprintln!("{:?}", _decl),
			Array {
				type_qualifier_list,
				assignment_expr,
				has_static,
				has_ptr,
			} => (),
			ParameterTypeList(type_list) => {
				for param in type_list.parameter_list.iter_mut() {
					self.parameter_declaration(param);
				}
			}
			IdentifierList(_ident_list) => (),
		}
	}
	fn parameter_type_list(&mut self, list: &mut ParameterTypeList) {
		for param in list.parameter_list.iter_mut() {
			self.parameter_declaration(param);
		}
	}
	fn pointer(&mut self, ptr: &mut Pointer) {
		for qual in ptr.type_qualifier_list.iter_mut() {
			self.type_qualifier(qual);
		}
	}
	fn type_qualifier(&mut self, qual: &mut TypeQualifier) {
		use TypeQualifier::*;
		match qual {
			Const => (),
			Restrict => (),
			Volatile => (),
		}
	}
	fn parameter_declaration(&mut self, param: &mut ParameterDeclaration) {
		self.parameter_declarator(&mut param.parameter_declarator);
	}
	fn parameter_declarator(&mut self, param_decl: &mut ParameterDeclarator) {
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
	fn abstract_declarator(&mut self, decl: &mut AbstractDeclarator) {
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
	fn direct_abstract_declarator(&mut self, decl: &mut DirectAbstractDeclarator) {
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
	fn specifier_qualifier(&mut self, spec: &mut SpecifierQualifier) {
		use SpecifierQualifier::*;
		match spec {
			TypeSpecifier(ty) => todo!(),
			TypeQualifier(ty) => self.type_qualifier(ty),
		}
	}
	fn initializer_list(&mut self, list: &mut InitializerList) {
		for (desig, ref mut init) in list.0.iter_mut() {
			if let Some(ref mut desig) = desig {
				self.designation(desig);
			}
			self.initializer(init);
		}
	}
	fn designation(&mut self, desig: &mut Designation) {
		for ref mut desig in desig.0.iter_mut() {
			self.designator(desig)
		}
	}
	fn designator(&mut self, desig: &mut Designator) {
		use Designator::*;
		match desig {
			ConstantExpr(expr) => self.expr(expr),
			Dot(_) => (),
		}
	}
}
