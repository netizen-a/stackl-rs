pub enum DataType {
	Struct(Vec<DataType>),
	Union(Vec<DataType>),
	Void,
	UInt(u32),
	SInt(u32),
}

pub enum StorageClass {
	Extern,
	Static,
}
