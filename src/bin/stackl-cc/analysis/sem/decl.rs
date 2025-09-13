use crate::analysis::sem::Namespace;
use crate::analysis::sem::StorageClass;
use crate::analysis::sem::SymbolTableEntry;
use crate::analysis::syn::*;
use crate::analysis::tok;
use crate::data_types::DataType;
use crate::data_types::MemberType;
use crate::data_types::ScalarType;
use crate::diagnostics as diag;

const SIGNED_STR: &str = "signed";
const UNSIGNED_STR: &str = "unsigned";
const FLOAT_STR: &str = "float";
const DOUBLE_STR: &str = "double";
const LONG_STR: &str = "long";
const CHAR_STR: &str = "char";
const VOID_STR: &str = "void";
const SHORT_STR: &str = "void";
const BOOL_STR: &str = "_Bool";
const LONG_LONG_STR: &str = "long long";
const STRUCT_STR: &str = "struct";

impl super::SemanticParser<'_> {
	pub(super) fn function_definition(&mut self, decl: &mut FunctionDefinition) {
		self.specifiers(&mut decl.specifiers);
		let _ = decl.declarator;
		self.symtab.increase_scope();
		for declaration in decl.declaration_list.iter_mut() {
			self.declaration(declaration, StorageClass::Auto);
		}
		self.compound_stmt(&mut decl.compound_stmt);
		self.symtab.decrease_scope();
	}
	pub(super) fn declaration(&mut self, decl: &mut Declaration, default_sc: StorageClass) {
		let (maybe_sc, maybe_ty) = self.specifiers(&mut decl.specifiers);
		let storage_class = maybe_sc.unwrap_or(default_sc);

		for ref mut init_decl in decl.init_declarator_list.iter_mut() {
			if let Some(data_type) = &maybe_ty {
			} else {
				let diag = diag::Diagnostic::error(
					diag::DiagKind::ImplicitInt(init_decl.identifier.name.clone()),
					init_decl.identifier.span.clone(),
				);
				self.diagnostics.push(diag);
			}
			//self.init_declarator(init_decl);
			//let ident = init_decl.identifier;
			let _ = init_decl.declarator;
			if let Some(ref mut init) = init_decl.initializer {
				self.initializer(init);
			}
		}
		// let entry = SymbolTableEntry { data_type, is_incomplete: false, linkage:  };
		// self.symtab.insert(Namespace::Ordinary(ident.name), entry);
	}
	fn specifiers(
		&mut self,
		specifiers: &mut Specifiers,
	) -> (Option<StorageClass>, Option<DataType>) {
		let mut storage_class = None;
		for (i, storage_class_specifier) in specifiers.storage_classes.iter().enumerate() {
			if i > 0 {
				let diag = diag::Diagnostic::error(
					diag::DiagKind::MultStorageClasses,
					storage_class_specifier.span.clone(),
				);
				self.diagnostics.push(diag);
				storage_class = None;
			} else {
				storage_class = Some(storage_class_specifier.storage_class);
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
		let mut data_type: Option<DataType> = None;
		let mut long_count = 0;
		for type_spec in specifiers.type_specifiers.iter_mut() {
			match type_spec {
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
						None => data_type = Some(DataType::Scalar(ScalarType::I8)),
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
						None => data_type = Some(DataType::Scalar(ScalarType::I16)),
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
					None => data_type = Some(DataType::Scalar(ScalarType::I32)),
				},
				TypeSpecifier::Long(span) => {
					long_count += 1;
					if long_count > 2 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::TooLong,
							span.clone(),
						));
					}
					match &data_type {
						Some(data_type) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								data_type.to_string(),
							),
							span.clone(),
						)),
						None | Some(DataType::Scalar(ScalarType::I32)) => {
							// do nothing
						}
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
					match data_type {
						Some(_) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::MultipleTypes,
							span.clone(),
						)),
						None => data_type = Some(DataType::Scalar(ScalarType::Float)),
					}
					if long_count > 0 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_STR.to_owned(),
								FLOAT_STR.to_owned(),
							),
							span.clone(),
						));
					}
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
					match data_type {
						Some(_) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::MultipleTypes,
							span.clone(),
						)),
						None => data_type = Some(DataType::Scalar(ScalarType::Double)),
					}
					if long_count > 1 {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								LONG_LONG_STR.to_owned(),
								DOUBLE_STR.to_owned(),
							),
							span.clone(),
						));
					}
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
					match &data_type {
						Some(DataType::Scalar(ScalarType::Double)) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									SIGNED_STR.to_owned(),
									DOUBLE_STR.to_owned(),
								),
								span.clone(),
							))
						}
						Some(DataType::Scalar(ScalarType::Float)) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									SIGNED_STR.to_owned(),
									FLOAT_STR.to_owned(),
								),
								span.clone(),
							))
						}
						Some(name) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(SIGNED_STR.to_owned(), name.to_string()),
							span.clone(),
						)),
						Some(DataType::Scalar(_)) | None => {
							// do nothing
						}
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
					match &data_type {
						Some(DataType::Scalar(ScalarType::Double)) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									UNSIGNED_STR.to_owned(),
									DOUBLE_STR.to_owned(),
								),
								span.clone(),
							))
						}
						Some(DataType::Scalar(ScalarType::Float)) => {
							self.diagnostics.push(diag::Diagnostic::error(
								diag::DiagKind::BothSpecifiers(
									UNSIGNED_STR.to_owned(),
									FLOAT_STR.to_owned(),
								),
								span.clone(),
							))
						}
						Some(name) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								UNSIGNED_STR.to_owned(),
								name.to_string(),
							),
							span.clone(),
						)),
						Some(DataType::Scalar(_)) | None => {
							// do nothing
						}
					}
				}
				TypeSpecifier::Bool(span) => {
					match data_type {
						Some(_) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::MultipleTypes,
							span.clone(),
						)),
						None => data_type = Some(DataType::Scalar(ScalarType::Bool)),
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
				TypeSpecifier::StructOrUnionSpecifier(StructOrUnionSpecifier {
					struct_or_union,
					struct_declaration_list,
					..
				}) => {
					let span = struct_or_union.span.clone();
					let mut members = vec![];
					for decl in struct_declaration_list.iter_mut() {
						let k = self.struct_declaration(decl);
						members.push(k);
					}
					if data_type.is_some() {
						self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::MultipleTypes,
							span.clone(),
						));
					} else if let tok::Keyword::Struct = struct_or_union.keyword {
						data_type = Some(DataType::Struct(vec![]));
					} else if let tok::Keyword::Union = struct_or_union.keyword {
						data_type = Some(DataType::Union(vec![]));
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
					match is_signed {
						Some(true) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								SIGNED_STR.to_owned(),
								STRUCT_STR.to_owned(),
							),
							span.clone(),
						)),
						Some(false) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::BothSpecifiers(
								UNSIGNED_STR.to_owned(),
								STRUCT_STR.to_owned(),
							),
							span.clone(),
						)),
						None => {
							// do nothing
						}
					}
				}
				TypeSpecifier::EnumSpecifier(EnumSpecifier { tag_span, .. }) => {
					let span = tag_span.clone();
					match data_type {
						Some(_) => self.diagnostics.push(diag::Diagnostic::error(
							diag::DiagKind::MultipleTypes,
							span.clone(),
						)),
						None => data_type = Some(DataType::Scalar(ScalarType::I8)),
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
				TypeSpecifier::TypedefName { .. } => todo!("typedef"),
			}
		}
		if let Some(DataType::Scalar(ref mut scalar)) = &mut data_type {
			if let ScalarType::I32 = scalar {
				match long_count {
					1 => *scalar = ScalarType::I64,
					2 => *scalar = ScalarType::I128,
					_ => {}
				}
			}
			if let Some(is_signed) = is_signed {
				scalar.set_signedness(is_signed);
			}
		}
		(storage_class, data_type)
	}
	fn init_declarator(&mut self, decl: &mut InitDeclarator) {
		let _ = decl.identifier;
		let _ = decl.declarator;
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

	fn struct_declaration(&mut self, struct_decl: &mut StructDeclaration) {
		self.specifiers(&mut struct_decl.specifiers);
		for decl in struct_decl.struct_declaration_list.iter_mut() {
			self.struct_declarator(decl);
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
			TypeQualifierKind::Restrict => (),
			TypeQualifierKind::Volatile => (),
		}
	}
	fn parameter_declaration(&mut self, param: &mut ParameterDeclaration) {
		self.specifiers(&mut param.specifiers);
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
