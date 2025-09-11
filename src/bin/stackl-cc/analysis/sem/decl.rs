use crate::analysis::syn::*;
use crate::analysis::tok;
use crate::data_types::DataType;
use crate::data_types::Scalar;
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
		const SIGNED_STR: &str = "signed";
		const UNSIGNED_STR: &str = "unsigned";
		const FLOAT_STR: &str = "float";
		const DOUBLE_STR: &str = "double";
		const LONG_STR: &str = "long";
		const CHAR_STR: &str = "char";
		const VOID_STR: &str = "void";
		const SHORT_STR: &str = "void";
		const BOOL_STR: &str = "_Bool";
		for (i, storage_class) in specifiers.storage_classes.iter().enumerate() {
			if i > 0 {
				let diag = diag::Diagnostic::error(
					diag::DiagKind::MultStorageClasses,
					storage_class.span.clone(),
				);
				self.diagnostics.push(diag);
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

		let mut is_signed: Option<bool> = None;
		let mut is_double: Option<bool> = None;
		let mut data_type: Option<DataType> = None;
		let mut long_count = 0;
		for ty in specifiers.type_specifiers.iter() {
			match ty {
				TypeSpecifier::Void(span) => {
					match data_type {
						Some(_) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::MultipleTypes,
							span.clone(),
						)),
						None => data_type = Some(DataType::Void),
					}
					if long_count > 0 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								VOID_STR.to_owned(),
							),
							span.clone(),
						));
					}
				}
				TypeSpecifier::Char(span) => {
					match data_type {
						Some(_) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::MultipleTypes,
							span.clone(),
						)),
						None => data_type = Some(DataType::Scalar(Scalar::I8)),
					}
					if long_count > 0 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								CHAR_STR.to_owned(),
							),
							span.clone(),
						));
					}
				}
				TypeSpecifier::Short(span) => {
					match data_type {
						Some(_) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::MultipleTypes,
							span.clone(),
						)),
						None => data_type = Some(DataType::Scalar(Scalar::I16)),
					}
					if long_count > 0 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								SHORT_STR.to_owned(),
							),
							span.clone(),
						));
					}
				}
				TypeSpecifier::Int(span) => match data_type {
					Some(_) => self.diagnostics.push(diag::Diagnostic::error(
						diag::DiagKind::MultipleTypes,
						span.clone(),
					)),
					None => data_type = Some(DataType::Scalar(Scalar::I32)),
				},
				TypeSpecifier::Long(span) => {
					long_count += 1;
					match &data_type {
						Some(data_type) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								data_type.to_string(),
							),
							span.clone(),
						)),
						None | Some(DataType::Scalar(Scalar::I32)) => {
							// do nothing
						},
					}
				}
				TypeSpecifier::Float(span) => {
					match is_signed {
						Some(true) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								SIGNED_STR.to_owned(),
								FLOAT_STR.to_owned(),
							),
							span.clone(),
						)),
						Some(false) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								UNSIGNED_STR.to_owned(),
								FLOAT_STR.to_owned(),
							),
							span.clone(),
						)),
						None => {
							// do nothing
						}
					}
					is_double = Some(false);
				}
				TypeSpecifier::Double(span) => {
					match is_signed {
						Some(true) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								SIGNED_STR.to_owned(),
								DOUBLE_STR.to_owned(),
							),
							span.clone(),
						)),
						Some(false) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								UNSIGNED_STR.to_owned(),
								DOUBLE_STR.to_owned(),
							),
							span.clone(),
						)),
						None => {
							// do nothing
						}
					}
					is_double = Some(true);
				}
				TypeSpecifier::Signed(span) => {
					match is_signed {
						Some(true) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::DuplicateSpecifier(SIGNED_STR.to_owned()),
							span.clone(),
						)),
						Some(false) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								SIGNED_STR.to_owned(),
								UNSIGNED_STR.to_owned(),
							),
							span.clone(),
						)),
						None => is_signed = Some(true),
					}
					match is_double {
						Some(true) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								SIGNED_STR.to_owned(),
								DOUBLE_STR.to_owned(),
							),
							span.clone(),
						)),
						Some(false) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								SIGNED_STR.to_owned(),
								FLOAT_STR.to_owned(),
							),
							span.clone(),
						)),
						None => {}
					}
				}
				TypeSpecifier::Unsigned(span) => {
					match is_signed {
						Some(true) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								SIGNED_STR.to_owned(),
								UNSIGNED_STR.to_owned(),
							),
							span.clone(),
						)),
						Some(false) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::DuplicateSpecifier(UNSIGNED_STR.to_owned()),
							span.clone(),
						)),
						None => is_signed = Some(false),
					}
					match is_double {
						Some(true) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								UNSIGNED_STR.to_owned(),
								DOUBLE_STR.to_owned(),
							),
							span.clone(),
						)),
						Some(false) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								UNSIGNED_STR.to_owned(),
								FLOAT_STR.to_owned(),
							),
							span.clone(),
						)),
						None => {}
					}
				}
				TypeSpecifier::Bool(span) => {
					match data_type {
						Some(_) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::MultipleTypes,
							span.clone(),
						)),
						None => data_type = Some(DataType::Scalar(Scalar::Bool)),
					}
					if long_count > 0 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								BOOL_STR.to_owned(),
							),
							span.clone(),
						));
					}
					match is_signed {
						Some(true) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								SIGNED_STR.to_owned(),
								BOOL_STR.to_owned(),
							),
							span.clone(),
						)),
						Some(false) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								UNSIGNED_STR.to_owned(),
								BOOL_STR.to_owned(),
							),
							span.clone(),
						)),
						None => {
							// do nothing
						}
					}
				}
				TypeSpecifier::StructOrUnionSpecifier(_) => {}
				TypeSpecifier::EnumSpecifier(_) => {}
				TypeSpecifier::TypedefName { .. } => {}
			}
		}
	}
	fn init_declarator(&mut self, decl: &mut InitDeclarator) {
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
