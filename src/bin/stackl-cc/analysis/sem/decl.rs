use crate::analysis::syn::*;
use crate::analysis::tok;
use crate::diagnostics as diag;

impl super::SemanticParser<'_> {
	pub(super) fn function_definition(&mut self, decl: &mut FunctionDefinition) {
		self.declaration_specifiers(&decl.specifiers);
		for declaration in decl.declaration_list.iter_mut() {
			self.declaration(declaration);
		}
		self.compound_stmt(&mut decl.compound_stmt);
	}
	pub(super) fn declaration(&mut self, decl: &mut Declaration) {
		self.declaration_specifiers(&decl.specifiers);
		for ref mut init_decl in decl.init_declarator_list.iter_mut() {
			self.init_declarator(init_decl);
		}
	}
	fn declaration_specifiers(&mut self, specifiers: &DeclarationSpecifiers) {
		if specifiers.storage_classes.len() > 1 {
			for (i, storage_class) in specifiers.storage_classes.iter().enumerate() {
				if i > 0 {
					let diag = diag::Diagnostic::error(
						diag::DiagKind::MultStorageClasses,
						storage_class.span.clone(),
					);
					self.diagnostics.push(diag);
				}
			}
		}
		for (i, restrict_span) in specifiers.restrict_list.iter().enumerate() {
			let diag = if i == 0 {
				diag::Diagnostic::error(diag::DiagKind::InvalidRestrict, restrict_span.clone())
			} else {
				diag::Diagnostic::warn(
					diag::DiagKind::DuplicateSpecifier("restrict".to_owned()),
					restrict_span.clone(),
				)
			};
			self.diagnostics.push(diag);
		}
	}
	fn init_declarator(&mut self, decl: &mut InitDeclarator) {
		// self.declarator(&mut decl.declarator);
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
		// if let Some(ref mut decl) = struct_decl.declarator {
		// 	self.declarator(decl)
		// }
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
	fn direct_declarator(&mut self, direct_decl: &mut DirectDeclarator) {
		use DirectDeclarator::*;
		match direct_decl {
			//Declarator(_decl) => {},
			Pointer(_ptr) => {}
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
	fn type_qualifier(&mut self, qual: &mut TypeQualifier) {
		match qual.kind {
			TypeQualifierKind::Const => (),
			TypeQualifierKind::Restrict(_) => (),
			TypeQualifierKind::Volatile => (),
		}
	}
	fn parameter_declaration(&mut self, param: &mut ParameterDeclaration) {
		self.declaration_specifiers(&param.specifiers);
		self.parameter_declarator(&mut param.parameter_declarator);
	}
	fn parameter_declarator(&mut self, param_decl: &mut ParameterDeclarator) {
		use ParameterDeclarator::*;
		match param_decl {
			Declarator(decl) => {}
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
			Pointer(ptr) => {
				// pointer
			}
			DirectAbstractDeclarator {
				pointer,
				direct_abstract_declarator,
			} => {
				// pointer
				for declarator in direct_abstract_declarator {
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
