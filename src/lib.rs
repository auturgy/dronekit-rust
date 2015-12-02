extern crate xml;

use std::fs::File;
use std::io::BufReader;
use std::default::Default;

use xml::reader::{EventReader, XmlEvent};

fn indent(size: usize) -> String {
    const INDENT: &'static str = "    ";
    (0..size).map(|_| INDENT)
             .fold(String::with_capacity(size*INDENT.len()), |r, s| r + s)
}

#[derive(Debug, PartialEq, Clone)]
struct MavEnum {
    name: String,
    description: Option<String>,
    entries: Vec<MavEnumEntry>
}

impl Default for MavEnum {
    fn default() -> MavEnum {
        MavEnum {
            name: "".into(),
            description: None,
            entries: vec![],
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
struct MavEnumEntry {
    value: i32,
    name: String,
    description: Option<String>,
    params: Option<Vec<String>>,
}

impl Default for MavEnumEntry {
    fn default() -> MavEnumEntry {
        MavEnumEntry {
            value: 0,
            name: "".into(),
            description: None,
            params: None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
struct MavMessage {
    id: u8,
    name: String,
    description: Option<String>,
    fields: Vec<MavField>,
}

impl Default for MavMessage {
    fn default() -> MavMessage {
        MavMessage {
            id: 0,
            name: "".into(),
            description: None,
            fields: vec![],
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum MavType {
    UInt8MavlinkVersion,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Int8,
    Int16,
    Int32,
    Int64,
    Char,
    Float,
    Double,
    Buffer(u32),
}

fn parse_type(s: &str) -> Option<MavType> {
    use MavType::*;
    match s {
        "uint8_t" => Some(UInt8),
        "uint16_t" => Some(UInt16),
        "uint32_t" => Some(UInt32),
        "uint64_t" => Some(UInt64),
        "int8_t" => Some(Int8),
        "int16_t" => Some(Int16),
        "int32_t" => Some(Int32),
        "int64_t" => Some(Int64),
        "char" => Some(Char),
        "float" => Some(Float),
        "Double" => Some(Double),
        _ => {
            if s.starts_with("uint8_t[") && s.ends_with("]") {
                match s[("uint8_t[".len())..(s.len()-1)].parse::<u32>() {
                    Ok(val) => Some(Buffer(val)),
                    _ => None,
                }
            } else {
                None
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
struct MavField {
    mavtype: MavType,
    name: String,
    description: Option<String>,
    enumtype: Option<String>,
}

impl Default for MavField {
    fn default() -> MavField {
        MavField {
            mavtype: MavType::UInt8,
            name: "".into(),
            description: None,
            enumtype: None,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum MavXmlElement {
    Mavlink,
    Include,
    Enums,
    Enum,
    Entry,
    Description,
    Param,
    Messages,
    Message,
    Field,
}

fn identify_element(s: &str) -> Option<MavXmlElement> {
    use MavXmlElement::*;
    match s {
        "mavlink" => Some(Mavlink),
        "include" => Some(Include),
        "enums" => Some(Enums),
        "enum" => Some(Enum),
        "entry" => Some(Entry),
        "description" => Some(Description),
        "param" => Some(Param),
        "messages" => Some(Messages),
        "message" => Some(Message),
        "field" => Some(Field),
        _ => None,
    }
}

fn is_valid_parent(p: Option<MavXmlElement>, s: MavXmlElement) -> bool {
    use MavXmlElement::*;
    match s {
        Mavlink => p == None,
        Include => p == Some(Mavlink),
        Enums => p == Some(Mavlink),
        Enum => p == Some(Enums),
        Entry => p == Some(Enum),
        Description => p == Some(Entry) || p == Some(Message) || p == Some(Enum),
        Param => p == Some(Entry),
        Messages => p == Some(Mavlink),
        Message => p == Some(Messages),
        Field => p == Some(Message),
    }
}


#[test]
fn test_all() {
    let file = File::open("solo.xml").unwrap();
    let file = BufReader::new(file);

    let mut stack: Vec<MavXmlElement> = vec![];

    let mut messages: Vec<MavMessage> = vec![];
    let mut enums: Vec<MavEnum> = vec![];

    let mut field: MavField = Default::default();
    let mut message: MavMessage = Default::default();
    let mut mavenum: MavEnum = Default::default();
    let mut entry: MavEnumEntry = Default::default();
    let mut paramid: Option<usize> = None;

    let parser = EventReader::new(file);
    let mut depth = 0;
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, attributes: attrs, .. }) => {
                let id = match identify_element(&name.to_string()) {
                    None => {
                        panic!("unexpected element {:?}", name);
                    }
                    Some(kind) => kind,
                };

                if !is_valid_parent(match stack.last().clone() {
                    Some(arg) => Some(arg.clone()),
                    None => None,
                }, id.clone()) {
                    panic!("not valid parent {:?} of {:?}", stack.last(), id);
                }

                match id {
                    MavXmlElement::Message => {
                        message = Default::default();
                    },
                    MavXmlElement::Field => {
                        field = Default::default();
                    },
                    MavXmlElement::Enum => {
                        mavenum = Default::default();
                    },
                    MavXmlElement::Entry => {
                        entry = Default::default();
                    },
                    MavXmlElement::Param => {
                        paramid = None;
                    },
                    _ => ()
                }

                stack.push(id);
                // println!("{}+{:?}", indent(depth), id);

                for attr in attrs {
                    match stack.last() {
                        Some(&MavXmlElement::Enum) => {
                            match attr.name.local_name.clone().as_ref() {
                                "name" => {
                                    mavenum.name = attr.value.clone();
                                },
                                _ => (),
                            }
                        },
                        Some(&MavXmlElement::Entry) => {
                            match attr.name.local_name.clone().as_ref() {
                                "name" => {
                                    entry.name = attr.value.clone();
                                },
                                "value" => {
                                    entry.value = attr.value.parse::<i32>().unwrap();
                                },
                                _ => (),
                            }
                        },
                        Some(&MavXmlElement::Message) => {
                            match attr.name.local_name.clone().as_ref() {
                                "name" => {
                                    message.name = attr.value.clone();
                                },
                                "id" => {
                                    message.id = attr.value.parse::<u8>().unwrap();
                                },
                                _ => (),
                            }
                        },
                        Some(&MavXmlElement::Field) => {
                            match attr.name.local_name.clone().as_ref() {
                                "name" => {
                                    field.name = attr.value.clone();
                                },
                                "type" => {
                                    field.mavtype = parse_type(&attr.value).unwrap();
                                },
                                "enum" => {
                                    field.enumtype = Some(attr.value.clone());
                                },
                                _ => (),
                            }
                        },
                        Some(&MavXmlElement::Param) => {
                            if let None = entry.params {
                                entry.params = Some(vec![]);
                            }
                            match attr.name.local_name.clone().as_ref() {
                                "index" => {
                                    paramid = Some(attr.value.parse::<usize>().unwrap());
                                },
                                _ => (),
                            }
                        },
                        _ => (),
                    }
                }

                depth += 1;
            }
            Ok(XmlEvent::Characters(s)) => {
                use MavXmlElement::*;
                match (stack.last(), stack.get(stack.len() - 2)) {
                    (Some(&Description), Some(&Message)) => {
                        message.description = Some(s);
                    },
                    (Some(&Field), Some(&Message)) => {
                        field.description = Some(s);
                    },
                    (Some(&Description), Some(&Enum)) => {
                        mavenum.description = Some(s);
                    },
                    (Some(&Description), Some(&Entry)) => {
                        entry.description = Some(s);
                    },
                    (Some(&Param), Some(&Entry)) => {
                        if let Some(ref mut params) = entry.params {
                            params.insert(paramid.unwrap() - 1, s);
                        }
                    },
                    (Some(&Include), Some(&Mavlink)) => {
                        println!("TODO: include {:?}", s);
                    },
                    data => {
                        panic!("unexpected text data {:?} reading {:?}", data, s);
                    },
                }
            }
            Ok(XmlEvent::EndElement { name }) => {
                match stack.last() {
                    Some(&MavXmlElement::Field) => {
                        message.fields.push(field.clone())
                    },
                    Some(&MavXmlElement::Message) => {
                        // println!("message: {:?}", message);
                        messages.push(message.clone());
                    },
                    Some(&MavXmlElement::Entry) => {
                        mavenum.entries.push(entry.clone());
                    },
                    Some(&MavXmlElement::Enum) => {
                        println!("enum: {:?}", mavenum);
                        enums.push(mavenum.clone());
                    },
                    _ => (),
                }
                stack.pop();
                depth -= 1;
                // println!("{}-{}", indent(depth), name);
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }
}
