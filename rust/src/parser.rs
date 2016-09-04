use types::*;
use parser_types::*;
use util;
use std::mem;
use std::os::raw::c_char;

extern "C" {
    fn psyc_parse_state_init(state: *mut PsycParseState, flags: u8);
    fn psyc_parse_buffer_set(state: *mut PsycParseState, buffer: *const c_char, length: usize);
    fn psyc_parse_list_state_init(state: *mut PsycParseListState);
    fn psyc_parse_list_buffer_set(state: *mut PsycParseListState, buffer: *const c_char, length: usize);
    fn psyc_parse_cursor(state: *const PsycParseState) -> usize;
    fn psyc_parse_remaining_length(state: *const PsycParseState) -> usize;
    fn psyc_parse(state: *mut PsycParseState,
                  oper: *mut c_char,
                  name: *mut PsycString,
                  value: *mut PsycString)
                  -> PsycParseRC;

    fn psyc_parse_list(state: *mut PsycParseListState,
                       list_type: *mut PsycString,
                       elem: *mut PsycString)
                       -> PsycParseListRC;
}

pub struct PsycParser {
    state: PsycParseState
}

pub struct PsycListParser {
    state: PsycParseListState
}

#[derive(Debug, PartialEq)]
pub enum PsycParserResult<'a> {
    StateSync,
    StateReset,
    Complete,
    InsufficientData,
    RoutingModifier {
        operator: char,
        name: &'a [u8],
        value: &'a [u8]
    },
    EntityModifier {
        operator: char,
        name: &'a [u8],
        value: &'a [u8]
    },
    EntityModifierStart {
        operator: char,
        name: &'a [u8],
        value_part: &'a [u8]
    },
    EntityModifierCont {
        value_part: &'a [u8]
    },
    EntityModifierEnd {
        value_part: &'a [u8]
    },
    Body {
        name: &'a [u8],
        value: &'a [u8]
    },
    BodyStart {
        name: &'a [u8],
        value_part: &'a [u8]
    },
    BodyCont {
        value_part: &'a [u8]
    },
    BodyEnd {
        value_part: &'a [u8]
    }
}

#[derive(Debug, PartialEq)]
pub enum PsycListParserResult<'a> {
    Complete,
    InsufficientData,
    ListElement {
        value: &'a [u8]
    },
    ListElementStart {
        value_part: &'a [u8]
    },
    ListElementCont {
        value_part: &'a [u8]
    },
    ListElementEnd {
        value_part: &'a [u8]
    }
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub enum PsycParserError {
    NoModifierLength = PsycParseRC::PSYC_PARSE_ERROR_MOD_NO_LEN as _,
    NoContentLength = PsycParseRC::PSYC_PARSE_ERROR_NO_LEN as _,
    NoEndDelimiter = PsycParseRC::PSYC_PARSE_ERROR_END as _,
    NoNewlineAfterMethod = PsycParseRC::PSYC_PARSE_ERROR_METHOD as _,
    NoNewlineAfterModifier = PsycParseRC::PSYC_PARSE_ERROR_MOD_NL as _,
    InvalidModifierLength = PsycParseRC::PSYC_PARSE_ERROR_MOD_LEN as _,
    NoTabBeforeModifierValue = PsycParseRC::PSYC_PARSE_ERROR_MOD_TAB as _,
    NoModifierName = PsycParseRC::PSYC_PARSE_ERROR_MOD_NAME as _,
    NoNewlineAfterContentLength = PsycParseRC::PSYC_PARSE_ERROR_LENGTH as _,
    GenericError = PsycParseRC::PSYC_PARSE_ERROR as _,
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub enum PsycListParserError {
    NoElementLength = PsycParseListRC::PSYC_PARSE_LIST_ERROR_ELEM_NO_LEN as _,
    InvalidElementLength = PsycParseListRC::PSYC_PARSE_LIST_ERROR_ELEM_LENGTH as _,
    InvalidElementType = PsycParseListRC::PSYC_PARSE_LIST_ERROR_ELEM_TYPE as _,
    InvalidElementStart = PsycParseListRC::PSYC_PARSE_LIST_ERROR_ELEM_START as _,
    InvalidType = PsycParseListRC::PSYC_PARSE_LIST_ERROR_TYPE as _,
    GenericError = PsycParseListRC::PSYC_PARSE_LIST_ERROR as _,
}

impl PsycParser {
    /// Create a PsycParser
    pub fn new() -> Self {
        let mut state: PsycParseState;
        unsafe {
            state = mem::uninitialized();
            let state_ptr = &mut state as *mut PsycParseState;
            psyc_parse_state_init(state_ptr, PsycParseFlag::PSYC_PARSE_ALL as u8)
        }
        PsycParser {
            state: state
        }
    }

    /// Parse the buffer previously set by set_buffer. Call repeatedly until the
    /// result is PsycParserResult::Complete or a PsycParserError.
    pub fn parse<'a>(&mut self, buffer: &'a [u8]) -> Result<PsycParserResult<'a>, PsycParserError> {
        let state_ptr = &mut self.state as *mut PsycParseState;
        let buffer_ptr = buffer.as_ptr() as *const c_char;
        let mut operator = '\0';
        let mut name: PsycString;
        let mut value: PsycString;
        unsafe {
            if buffer_ptr != self.state.buffer.data ||
               buffer.len() != self.state.buffer.length {
                psyc_parse_buffer_set(state_ptr, buffer_ptr, buffer.len())
            }
            name = mem::uninitialized();
            value = mem::uninitialized();
            let operator_ptr = &mut operator as *mut char as *mut c_char;
            let name_ptr = &mut name as *mut PsycString;
            let value_ptr = &mut value as *mut PsycString;
            let parse_result = psyc_parse(state_ptr, operator_ptr, name_ptr, value_ptr);
            match parse_result {
                PsycParseRC::PSYC_PARSE_STATE_RESYNC =>
                    Ok(PsycParserResult::StateSync),

                PsycParseRC::PSYC_PARSE_STATE_RESET =>
                    Ok(PsycParserResult::StateReset),

                PsycParseRC::PSYC_PARSE_COMPLETE =>
                    Ok(PsycParserResult::Complete),

                PsycParseRC::PSYC_PARSE_INSUFFICIENT =>
                    Ok(PsycParserResult::InsufficientData),

                PsycParseRC::PSYC_PARSE_ROUTING => {
                    let result = PsycParserResult::RoutingModifier {
                        operator: operator,
                        name: util::cstring_to_slice(name.data, name.length),
                        value: util::cstring_to_slice(value.data, value.length)
                    };
                    Ok(result)
                },

                PsycParseRC::PSYC_PARSE_ENTITY => {
                    let result = PsycParserResult::EntityModifier {
                        operator: operator,
                        name: util::cstring_to_slice(name.data, name.length),
                        value: util::cstring_to_slice(value.data, value.length)
                    };
                    Ok(result)
                },

                PsycParseRC::PSYC_PARSE_ENTITY_START => {
                    let result = PsycParserResult::EntityModifierStart {
                        operator: operator,
                        name: util::cstring_to_slice(name.data, name.length),
                        value_part: util::cstring_to_slice(value.data, value.length)
                    };
                    Ok(result)
                },

                PsycParseRC::PSYC_PARSE_ENTITY_CONT => {
                    let result = PsycParserResult::EntityModifierCont {
                        value_part: util::cstring_to_slice(value.data, value.length)
                    };
                    Ok(result)
                },

                PsycParseRC::PSYC_PARSE_ENTITY_END => {
                    let result = PsycParserResult::EntityModifierEnd {
                        value_part: util::cstring_to_slice(value.data, value.length)
                    };
                    Ok(result)
                }

                PsycParseRC::PSYC_PARSE_BODY => {
                    let result = PsycParserResult::Body {
                        name: util::cstring_to_slice(name.data, name.length),
                        value: util::cstring_to_slice(value.data, value.length)
                    };
                    Ok(result)
                },

                PsycParseRC::PSYC_PARSE_BODY_START => {
                    let result = PsycParserResult::BodyStart {
                        name: util::cstring_to_slice(name.data, name.length),
                        value_part: util::cstring_to_slice(value.data, value.length)
                    };
                    Ok(result)
                },
                PsycParseRC::PSYC_PARSE_BODY_CONT => {
                    let result = PsycParserResult::BodyCont {
                        value_part: util::cstring_to_slice(value.data, value.length)
                    };
                    Ok(result)
                },

                PsycParseRC::PSYC_PARSE_BODY_END => {
                    let result = PsycParserResult::BodyEnd {
                        value_part: util::cstring_to_slice(value.data, value.length)
                    };
                    Ok(result)
                }

                _error => Err(mem::transmute(_error)),
            }
        }
    }
}

impl Parser for PsycParser {
    fn unparsed_position(&self) -> usize {
        unsafe {
            psyc_parse_cursor(&self.state as *const PsycParseState)
        }
    }

    fn unparsed_length(&self) -> usize {
        unsafe {
            psyc_parse_remaining_length(&self.state as *const PsycParseState)
        }
    }
}

impl PsycListParser {
    pub fn new() -> Self {
        let mut state: PsycParseListState;
        unsafe {
            state = mem::uninitialized();
            let state_ptr = &mut state as *mut PsycParseListState;
            psyc_parse_list_state_init(state_ptr)
        }
        PsycListParser {
            state: state
        }
    }

    pub fn parse<'a>(&mut self, buffer: &'a [u8]) -> Result<PsycListParserResult<'a>, PsycListParserError> {
        let state_ptr = &mut self.state as *mut PsycParseListState;
        let buffer_ptr = buffer.as_ptr() as *const c_char;
        let mut list_type: PsycString;
        let mut element: PsycString;
        unsafe {
            if buffer_ptr != self.state.buffer.data ||
               buffer.len() != self.state.buffer.length {
                psyc_parse_list_buffer_set(state_ptr, buffer_ptr, buffer.len())
            }
            list_type = mem::uninitialized();
            element = mem::uninitialized();
            let list_type_ptr = &mut list_type as *mut PsycString;
            let element_ptr = &mut element as *mut PsycString;
            loop {
                let parse_result = psyc_parse_list(state_ptr, list_type_ptr, element_ptr);
                println!("parse_result: {:?}", parse_result);
                println!("cursor: {}", self.state.cursor);
                match parse_result {
                    PsycParseListRC::PSYC_PARSE_LIST_END =>
                        return Ok(PsycListParserResult::Complete),
                    
                    PsycParseListRC::PSYC_PARSE_LIST_INSUFFICIENT =>
                        return Ok(PsycListParserResult::InsufficientData),

                    PsycParseListRC::PSYC_PARSE_LIST_ELEM_LAST |
                    PsycParseListRC::PSYC_PARSE_LIST_ELEM => {
                        let result = PsycListParserResult::ListElement {
                            value: util::cstring_to_slice(element.data, element.length)
                        };
                        return Ok(result)
                    },

                    PsycParseListRC::PSYC_PARSE_LIST_ELEM_START => {
                        let result = PsycListParserResult::ListElementStart {
                            value_part: util::cstring_to_slice(element.data, element.length)
                        };
                        return Ok(result)
                    },

                    PsycParseListRC::PSYC_PARSE_LIST_ELEM_CONT => {
                        let result = PsycListParserResult::ListElementCont {
                            value_part: util::cstring_to_slice(element.data, element.length)
                        };
                        return Ok(result)
                    },

                    PsycParseListRC::PSYC_PARSE_LIST_ELEM_END => {
                        let result = PsycListParserResult::ListElementEnd {
                            value_part: util::cstring_to_slice(element.data, element.length)
                        };
                        return Ok(result)
                    },

                    PsycParseListRC::PSYC_PARSE_LIST_TYPE => (),

                    _error => {
                        return Err(mem::transmute(_error))
                    },
                }
            }
        }
    }
}

impl Parser for PsycListParser {
    fn unparsed_position(&self) -> usize {
        self.state.cursor
    }

    fn unparsed_length(&self) -> usize {
        self.state.buffer.length - self.state.cursor
    }
}

pub trait Parser {
    /// copies the remaining unparsed bytes to the beginning of the given buffer.
    /// Returns the number of copied bytes. Must be called when parse() returned
    /// InsufficientData as Result.
    fn copy_unparsed_into_buffer<'a>(&self, buffer: &'a mut [u8]) -> usize {
        let unparsed_pos = self.unparsed_position();
        let unparsed_len = self.unparsed_length();
        if unparsed_pos != 0 {
            let copy_pos_second = unparsed_pos - unparsed_len;
            let (part1, part2) = buffer.split_at_mut(unparsed_len);
            part1.copy_from_slice(&part2[copy_pos_second .. copy_pos_second + unparsed_len]);
        }
        unparsed_len
    }

    fn unparsed_position(&self) -> usize;
    fn unparsed_length(&self) -> usize;
}
