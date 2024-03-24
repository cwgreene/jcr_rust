extern crate byteorder;
extern crate serde;

#[macro_use]
extern crate serde_derive;

use std::fs::File;
use std::io::Cursor;
use std::io::Read;
use std::env;

use serde::{Serialize, Deserialize};

use byteorder::{BigEndian, ReadBytesExt};

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let result : ClassFile = read_class_file("data/Test.class".to_string());
        assert_eq!(result.magic, 0xcafebabe);
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ClassFile {
    magic : u32,
    minor : u16,
    major : u16,
    constant_pool : Vec<ConstantPoolEntry>,
    access_flags : u16,
    this_class : u16,
    super_class : u16,
    interfaces : Vec<Interface>,
    fields : Vec<Field>,
    methods : Vec<Method>,
    attributes : Vec<Attribute>
}

#[derive(Serialize, Deserialize, Debug)]
struct Interface {
    interface_ix : u16
}

#[derive(Serialize, Deserialize, Debug)]
struct Attribute {
    attribute_name_ix : u16,
    info : Vec<u8>
}

#[derive(Serialize, Deserialize, Debug)]
struct Field {
    access_flags : u16,
    name_ix : u16,
    descriptor_ix : u16,
    attributes : Vec<Attribute>
}

#[derive(Serialize, Deserialize, Debug)]
struct Method {
    access_flags : u16,
    name_ix : u16,
    descriptor_ix : u16,
    attributes : Vec<Attribute>
}

const CONSTANT_CLASS : u8 = 7;
const CONSTANT_FIELDREF : u8 = 9;
const CONSTANT_METHODREF : u8 = 10;
const CONSTANT_INTERFACEMETHODREF : u8 = 11;
const CONSTANT_STRING : u8 = 8;
const CONSTANT_INTEGER : u8 = 3;
const CONSTANT_FLOAT : u8 = 4;
const CONSTANT_LONG : u8 = 5;
const CONSTANT_DOUBLE : u8 = 6;
const CONSTANT_NAMEANDTYPE : u8 = 12;
const CONSTANT_UTF8 : u8 = 1;
const CONSTANT_METHODHANDLE : u8 = 15;
const CONSTANT_METHODTYPE : u8 = 16;
const CONSTANT_INVOKEDYNAMIC : u8 = 18;

#[derive(Serialize, Deserialize, Debug)]
enum ConstantPoolEntry {
    ConstClass { name_ix : u16 },
    ConstFieldRef { class_ix : u16, name_and_type_ix : u16 },
    ConstMethodRef { class_ix : u16, name_and_type_ix : u16 },
    ConstInterfaceMethodRef { class_ix : u16, name_and_type_ix : u16 },
    ConstString { string_ix : u16 },
    ConstInt { int_word : u32 },
    ConstFloat { float_word : u32 },
    ConstLong { high_word : u32, low_word : u32},
    ConstDouble { high_word : u32, low_word : u32},
    ConstUtf8 { string : String },
    ConstNameAndType { name_ix : u16, descriptor_ix : u16},
    ConstMethodHandle { reference_kind : u8, reference_ix : u16 },
    ConstMethodType { descriptor_ix : u16 },
    ConstInvokeDynamic { boostrap_method_attr_ix : u16, name_and_type_ix : u16 },
    ConstUnused,
    ConstInvalid { tag: u8 }
}

fn read16(cursor : &mut Cursor<Vec<u8>>) -> u16 {
    cursor.read_u16::<BigEndian>().unwrap()
}

fn read32(cursor : &mut Cursor<Vec<u8>>) -> u32 {
    cursor.read_u32::<BigEndian>().unwrap()
}

pub fn read8(cursor : &mut Cursor<Vec<u8>>) -> u8 {
    cursor.read_u8().unwrap()
}

fn read_string(cursor : &mut Cursor<Vec<u8>>) -> String {
    let mut result = String::new();
    let size = read16(cursor);
    let mut buf = cursor.take(size as u64);
    buf.read_to_string(&mut result).unwrap();
    result
}

fn get_constant_pool_entry(crs : &mut Cursor<Vec<u8>>, tag : u8) -> ConstantPoolEntry {
    let result = match tag {
        CONSTANT_CLASS => ConstantPoolEntry::ConstClass { name_ix : read16(crs) },
        CONSTANT_FIELDREF => ConstantPoolEntry::ConstFieldRef {
            class_ix : read16(crs),
            name_and_type_ix: read16(crs)
        },
        CONSTANT_METHODREF => ConstantPoolEntry::ConstMethodRef {
            class_ix : read16(crs),
            name_and_type_ix: read16(crs)
        },
        CONSTANT_INTERFACEMETHODREF => ConstantPoolEntry::ConstInterfaceMethodRef {
            class_ix : read16(crs),
            name_and_type_ix: read16(crs)
        },
        CONSTANT_STRING => ConstantPoolEntry::ConstString { string_ix : read16(crs) },
        CONSTANT_INTEGER => ConstantPoolEntry::ConstInt { int_word : read32(crs) },
        CONSTANT_FLOAT => ConstantPoolEntry::ConstFloat { float_word : read32(crs) },
        CONSTANT_LONG => ConstantPoolEntry::ConstLong {
            high_word : read32(crs),
            low_word : read32(crs)
        },
        CONSTANT_DOUBLE => ConstantPoolEntry::ConstDouble {
            high_word : read32(crs),
            low_word : read32(crs)
        },
        CONSTANT_UTF8 => ConstantPoolEntry::ConstUtf8 { string : read_string(crs) },
        CONSTANT_METHODHANDLE => ConstantPoolEntry::ConstMethodHandle {
            reference_kind : read8(crs),
            reference_ix : read16(crs)
        },
        CONSTANT_NAMEANDTYPE => ConstantPoolEntry::ConstNameAndType {
            name_ix : read16(crs),
            descriptor_ix : read16(crs)
        },
        CONSTANT_METHODTYPE => ConstantPoolEntry::ConstMethodType { descriptor_ix : read16(crs) },
        CONSTANT_INVOKEDYNAMIC => ConstantPoolEntry::ConstInvokeDynamic {
            boostrap_method_attr_ix : read16(crs),
            name_and_type_ix : read16(crs)
        },
        _ => ConstantPoolEntry::ConstInvalid {tag: tag},
    };
    result
}

fn get_constant_pool(cursor : &mut Cursor<Vec<u8>>, cp_count : u16) -> Vec<ConstantPoolEntry> {
    let mut pool = Vec::with_capacity((cp_count - 1) as usize);
    let mut ix = 1;
    while ix < cp_count {
        let tag = read8(cursor);
        let entry = get_constant_pool_entry(cursor, tag);
        pool.push(entry);
        if tag == CONSTANT_LONG || tag == CONSTANT_DOUBLE {
            pool.push(ConstantPoolEntry::ConstUnused);
            ix += 2;
        } else {
            ix += 1;
        }
    }
    pool
}

fn get_methods(cursor : &mut Cursor<Vec<u8>>) -> Vec<Method> {
    let method_count = read16(cursor);
    let mut result = Vec::with_capacity(method_count as usize);
    let mut ix = 0;
    while ix < method_count {
        result.push(Method {
            access_flags : read16(cursor),
            name_ix : read16(cursor),
            descriptor_ix : read16(cursor),
            attributes : get_attributes(cursor)   
        });
        ix += 1;
    }
    result
}

fn get_attributes(cursor : &mut Cursor<Vec<u8>>) -> Vec<Attribute> {
    let attribute_count = read16(cursor);
    let mut result = Vec::with_capacity(attribute_count as usize);
    let mut ix = 0;
    while ix < attribute_count {
        let attribute_name_ix = read16(cursor);
        let length = read32(cursor);
        let mut info = Vec::new();
        cursor.take(length as u64).read_to_end(&mut info).unwrap();
        result.push(Attribute {
            attribute_name_ix : attribute_name_ix,
            info : info
        });
        ix += 1;
    }
    result
}

fn get_interfaces(cursor : &mut Cursor<Vec<u8>>) -> Vec<Interface> {
    let interface_count = read16(cursor);
    let mut result = Vec::with_capacity(interface_count as usize);
    let mut ix = 0;
    while ix < interface_count {
        result.push(Interface { interface_ix : read16(cursor) } );
        ix += 1;
    }
    result
}

fn get_fields(cursor : &mut Cursor<Vec<u8>>) -> Vec<Field> {
    let field_count = read16(cursor);
    let mut result = Vec::with_capacity(field_count as usize);
    let mut ix = 0;
    while ix < field_count {
        result.push(Field {
            access_flags : read16(cursor),
            name_ix : read16(cursor),
            descriptor_ix : read16(cursor),
            attributes : get_attributes(cursor)
        });
        ix += 1;
    }
    result
}

fn read_class_file (file_name : String) -> ClassFile {
    let mut f = File::open(file_name).expect("unable to open file.");
    let mut v = Vec::new();
    f.read_to_end(&mut v).unwrap();
    let mut cursor = Cursor::new(v);

    // Read in
    let magic = read32(&mut cursor);
    let minor = read16(&mut cursor);
    let major = read16(&mut cursor);
    let cp_count = read16(&mut cursor);
    let cp = get_constant_pool(&mut cursor, cp_count);
    let access_flags = read16(&mut cursor);
    let this_class = read16(&mut cursor);
    let super_class = read16(&mut cursor);
    let interfaces = get_interfaces(&mut cursor);
    let fields = get_fields(&mut cursor);
    let methods = get_methods(&mut cursor);
    let attributes = get_attributes(&mut cursor);

    ClassFile {
        magic : magic,
        minor : minor,
        major : major,
        constant_pool : cp,
        access_flags : access_flags,
        this_class : this_class,
        super_class : super_class,
        interfaces : interfaces,
        fields : fields,
        methods : methods,
        attributes : attributes
    }
}

fn main() {
    let mut json = false;
    for param in env::args().skip(1) {
        if param == "--json" {
            json = true;
        }
    }
    for file in env::args().skip(1) {
        if file == "--json" {
            continue;
        }
        let classfile = read_class_file(file.to_string());
        if json == true {
            let json_str = serde_json::to_string(&classfile).unwrap();
            println!("{}", json_str);
        } else {
            println!("{:?}", classfile);
        }
    }
}
