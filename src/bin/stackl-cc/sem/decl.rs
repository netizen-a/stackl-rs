use crate::syn::*;

impl super::SemanticParser {
	pub(super) fn declaration_specifier(&mut self, specifier: DeclarationSpecifier) {
		use DeclarationSpecifier::*;
		match specifier {
			StorageClassSpecifier(spec) => self.storage_class_specifier(spec),
			TypeSpecifier(spec) => self.type_specifier(spec),
			TypeQualifier(spec) => self.type_qualifier(spec),
			FunctionSpecifier(spec) => self.function_specifier(spec),
		}
	}
	pub(super) fn init_declarator(&mut self, _init_decl: InitDeclarator) {
		todo!("init-declarator")
	}
	pub(super) fn storage_class_specifier(&mut self, _spec: StorageClassSpecifier) {
		todo!("storage-class-specifier")
	}
	pub(super) fn type_specifier(&mut self, spec: TypeSpecifier) {
		use TypeSpecifier::*;
		match spec {
			Void => todo!("void"),
			Char => todo!("char"),
			Short => todo!("short"),
			Int => todo!("int"),
			Long => todo!("long"),
			Float => todo!("float"),
			Double => todo!("double"),
			Signed => todo!("signed"),
			Unsigned => todo!("unsigned"),
			Bool => todo!("_Bool"),
			StructOrUnionSpecifier(spec) => self.tag_specifier(spec),
			EnumSpecifier(spec) => self.enum_specifier(spec),
			TypedefName(_name) => todo!(),
		}
	}
	pub(super) fn enum_specifier(&mut self, _spec: EnumSpecifier) {
		todo!()
	}
	pub(super) fn tag_specifier(&mut self, _spec: StructOrUnionSpecifier) {
		todo!()
	}
	pub(super) fn function_specifier(&mut self, _spec: FunctionSpecifier) {
		todo!("function-specifier")
	}
	pub(super) fn declarator(&mut self, _declarator: Declarator) {
		todo!("declarator")
	}
	pub(super) fn type_qualifier(&mut self, _qual: TypeQualifier) {
		todo!("type-qualifier")
	}
}
