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
	pub(super) fn storage_class_specifier(&mut self, _spec: StorageClassSpecifier) {
		todo!("storage-class-specifier")
	}
	pub(super) fn type_specifier(&mut self, _spec: TypeSpecifier) {
		todo!("type-specifier")
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
