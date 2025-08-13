use crate::analysis::syn::*;
use crate::analysis::tok;

#[derive(Default)]
struct TypeSpecifierInfo {
	num_void: usize,
	num_char: usize,
	num_short: usize,
	num_int: usize,
	num_long: usize,
	num_float: usize,
	num_double: usize,
	num_signed: usize,
	num_unsigned: usize,
	num_bool: usize,
	num_struct_or_union: Vec<StructOrUnionSpecifier>,
	num_enum: Vec<EnumSpecifier>,
	typedef_name: Vec<tok::Ident>,
}


#[derive(Default)]
struct SpecifierInfo {
	storage_class: Option<StorageClassSpecifier>,
	type_info: TypeSpecifierInfo,
	// tq_info
	// fs_info
}

impl super::SemanticParser {
	pub(super) fn function_definition(&mut self, decl: &mut FunctionDefinition) {
		let mut spec_info = SpecifierInfo::default();
		for ref mut specifier in decl.declaration_specifiers.iter_mut() {
			self.declaration_specifier(specifier, &mut spec_info);
		}
		self.declarator(&mut decl.declarator);
		for declaration in decl.declaration_list.iter_mut() {
			self.declaration(declaration);
		}
		self.compound_stmt(&mut decl.compound_stmt);
	}
	pub(super) fn declaration(&mut self, decl: &mut Declaration) {
		let mut spec_info = SpecifierInfo::default();
		for ref mut spec in decl.declaration_specifiers.iter_mut() {
			self.declaration_specifier(spec, &mut spec_info);
		}
		for ref mut init_decl in decl.init_declarator_list.iter_mut() {
			self.init_declarator(init_decl);
		}
	}
	fn declaration_specifier(
		&mut self,
		specifier: &mut DeclarationSpecifier,
		info: &mut SpecifierInfo,
	) {
		use DeclarationSpecifier::*;
		match specifier {
			StorageClassSpecifier(spec) => {
				if let None = info.storage_class {
					info.storage_class = Some(*spec);
				} else {
					panic!("cannot have more than one storage spec")
				}
			}
			TypeSpecifier(spec) => self.type_specifier(spec, &mut info.type_info),
			TypeQualifier(spec) => self.type_qualifier(spec),
			FunctionSpecifier(spec) => self.function_specifier(spec),
		}
	}
	fn init_declarator(&mut self, decl: &mut InitDeclarator) {
		self.declarator(&mut decl.declarator);
		if let Some(ref mut init) = decl.initializer {
			self.initializer(init);
		}
	}
	fn type_specifier(&mut self, spec: &mut TypeSpecifier, info: &mut TypeSpecifierInfo) {
		use TypeSpecifier::*;
		match spec {
			Void => info.num_void += 1,
			Char => info.num_char += 1,
			Short => info.num_short += 1,
			Int => info.num_int += 1,
			Long => info.num_long += 1,
			Float => info.num_float += 1,
			Double => info.num_double += 1,
			Signed => info.num_signed += 1,
			Unsigned => info.num_unsigned += 1,
			Bool => info.num_bool += 1,
			StructOrUnionSpecifier(spec) => self.struct_or_union_specifier(spec),
			EnumSpecifier(spec) => self.enum_specifier(spec),
			TypedefName(_name) => todo!(),
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
		let mut spec_info = SpecifierInfo::default();
		for ref mut specifier in param.declaration_specifiers.iter_mut() {
			self.declaration_specifier(specifier, &mut spec_info);
		}
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
		let mut type_info = TypeSpecifierInfo::default();
		match spec {
			TypeSpecifier(ty) => self.type_specifier(ty, &mut type_info),
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
