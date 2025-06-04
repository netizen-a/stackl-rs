use crate::syn::*;

impl super::SemanticParser {
	pub(super) fn declaration(&mut self, decl: Declaration) {
		for spec in decl.declaration_specifiers {
			self.declaration_specifier(spec);
		}
		for init_decl in decl.init_declarator_list {
			self.init_declarator(init_decl);
		}
	}
	pub(super) fn declaration_specifier(&mut self, specifier: DeclarationSpecifier) {
		use DeclarationSpecifier::*;
		match specifier {
			StorageClassSpecifier(spec) => self.storage_class_specifier(spec),
			TypeSpecifier(spec) => self.type_specifier(spec),
			TypeQualifier(spec) => self.type_qualifier(spec),
			FunctionSpecifier(spec) => self.function_specifier(spec),
		}
	}
	pub(super) fn init_declarator(&mut self, decl: InitDeclarator) {
		self.declarator(decl.declarator);
		if let Some(init) = decl.initializer {
			self.initializer(init);
		}
	}
	pub(super) fn storage_class_specifier(&mut self, _spec: StorageClassSpecifier) {
		todo!("storage-class-specifier")
	}
	pub(super) fn type_specifier(&mut self, spec: TypeSpecifier) {
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
	pub(super) fn enum_specifier(&mut self, _spec: EnumSpecifier) {
		todo!("enum-specifier")
	}
	pub(super) fn enumerator(&mut self, enumerator: Enumerator) {
		if let Some(expr) = enumerator.constant_expr {
			self.expression(expr);
		}
	}

	pub(super) fn struct_declaration(&mut self, decl: StructDeclaration) {
		for spec in decl.specifier_qualifier_list {
			self.specifier_qualifier(spec);
		}
		for struct_decl in decl.struct_declaration_list {
			self.struct_declarator(struct_decl);
		}
		todo!("struct-declaration")
	}

	pub(super) fn struct_declarator(&mut self, struct_decl: StructDeclarator) {
		if let Some(decl) = struct_decl.declarator {
			self.declarator(decl)
		}
		if let Some(expr) = struct_decl.constant_expr {
			self.expression(expr);
		}
		// todo!("struct-declarator")
	}
	pub(super) fn struct_or_union_specifier(&mut self, _spec: StructOrUnionSpecifier) {
		todo!("struct-or-union-specifier")
	}
	pub(super) fn initializer(&mut self, init: Initializer) {
		use Initializer::*;
		match init {
			Expr(expr) => self.expression(expr),
			InitializerList(list) => self.initializer_list(list),
		}
	}
	pub(super) fn function_specifier(&mut self, _spec: FunctionSpecifier) {
		todo!("function-specifier")
	}
	pub(super) fn declarator(&mut self, decl: Declarator) {
		for ptr in decl.pointer {
			self.pointer(ptr);
		}
		for direct_decl in decl.direct_declarator {
			self.direct_declarator(direct_decl);
		}
	}
	pub(super) fn direct_declarator(&mut self, direct_decl: DirectDeclarator) {
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
				for param in type_list.parameter_list {
					self.parameter_declaration(param);
				}
			}
			IdentifierList(_ident_list) => (),
		}
	}
	pub(super) fn parameter_type_list(&mut self, list: ParameterTypeList) {
		for param in list.parameter_list {
			self.parameter_declaration(param);
		}
	}
	pub(super) fn pointer(&mut self, ptr: Pointer) {
		for qual in ptr.type_qualifier_list {
			self.type_qualifier(qual);
		}
	}
	pub(super) fn type_qualifier(&mut self, qual: TypeQualifier) {
		use TypeQualifier::*;
		match qual {
			Const => (),
			Restrict => (),
			Volatile => (),
		}
	}
	pub(super) fn parameter_declaration(&mut self, param: ParameterDeclaration) {
		for specifier in param.declaration_specifiers {
			self.declaration_specifier(specifier);
		}
		self.parameter_declarator(param.parameter_declarator);
	}
	pub(super) fn parameter_declarator(&mut self, param_decl: ParameterDeclarator) {
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
	pub(super) fn abstract_declarator(&mut self, decl: AbstractDeclarator) {
		use AbstractDeclarator::*;
		match decl {
			Pointer(ptr) => self.pointer(ptr),
			DirectAbstractDeclarator {
				pointer,
				direct_abstract_declarator,
			} => {
				todo!("direct-abstract-declarator")
			}
		}
	}
	pub(super) fn specifier_qualifier(&mut self, spec: SpecifierQualifier) {
		use SpecifierQualifier::*;
		match spec {
			TypeSpecifier(ty) => self.type_specifier(ty),
			TypeQualifier(ty) => self.type_qualifier(ty),
		}
	}
	pub(super) fn initializer_list(&mut self, list: InitializerList) {
		for (desig, init) in list.0 {
			if let Some(desig) = desig {
				self.designation(desig);
			}
			self.initializer(init);
		}
	}
	pub(super) fn designation(&mut self, desig: Designation) {
		for desig in desig.0 {
			self.designator(desig)
		}
	}
	pub(super) fn designator(&mut self, desig: Designator) {
		use Designator::*;
		match desig {
			ConstantExpr(expr) => self.expression(expr),
			Dot(_) => (),
		}
	}
}
