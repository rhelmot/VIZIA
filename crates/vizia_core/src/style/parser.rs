use std::fmt::Debug;

use cssparser::{
    self, AtRuleType, BasicParseError, BasicParseErrorKind, CowRcStr, DeclarationListParser,
    ParseError, ParseErrorKind, Parser, ParserInput, SourceLocation, Token,
};

use crate::animation::Transition;
use crate::style::color::Color;
use crate::style::property::Property;
use crate::style::selector::{Selector, SelectorRelation};
use crate::style::StyleRule;
use crate::style::*;
use vizia_id::GenerationalId;

#[derive(Clone)]
pub enum CustomParseError {
    InvalidLengthUnits(String),
    InvalidValue(String),
    UnrecognisedColorName(String),
    InvalidColorHex(String),
    InvalidStringName(String),
    UnrecognisedPseudoclass(String),
}

impl<'t> From<CustomParseError> for ParseError<'t, CustomParseError> {
    fn from(e: CustomParseError) -> Self {
        ParseError {
            kind: ParseErrorKind::Custom(e),
            location: SourceLocation { line: 0, column: 0 },
        }
    }
}

/// Type which describes errors produced by the css style parser.
pub struct StyleParseError<'t>(pub ParseError<'t, CustomParseError>);

impl<'t> std::fmt::Display for StyleParseError<'t> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error_message = match &self.0.kind {
            ParseErrorKind::Custom(custom_error) => {
                format!("{:?}", custom_error)
            }

            ParseErrorKind::Basic(basic_error) => {
                format!("{:?}", basic_error)
            }
        };

        write!(f, "Warning: {}", error_message)
    }
}

impl Debug for CustomParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CustomParseError::InvalidValue(error_string) => {
                write!(f, "Invalid value: {}", error_string)
            }

            CustomParseError::UnrecognisedPseudoclass(error_string) => {
                write!(f, "Unrecognised pseudoclass: {}", error_string)
            }

            CustomParseError::InvalidLengthUnits(error_string) => {
                write!(f, "Invalid length units: {}", error_string)
            }

            CustomParseError::InvalidStringName(error_string) => {
                write!(f, "Invalid string name: {}", error_string)
            }

            CustomParseError::UnrecognisedColorName(error_string) => {
                write!(f, "Unrecognised color name: {}", error_string)
            }

            CustomParseError::InvalidColorHex(error_string) => {
                write!(f, "Invalid color hex: {}", error_string)
            }
        }
    }
}

pub(crate) struct RuleParser;

impl RuleParser {
    pub fn new() -> Self {
        RuleParser {}
    }
}

impl<'i> cssparser::QualifiedRuleParser<'i> for RuleParser {
    type Prelude = Vec<Selector>;
    type QualifiedRule = StyleRule;
    type Error = CustomParseError;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
        let res = parse_selectors(input)?;
        Ok(res)
    }

    fn parse_block<'t>(
        &mut self,
        selectors: Self::Prelude,
        _location: SourceLocation,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::QualifiedRule, ParseError<'i, Self::Error>> {
        let decl_parser = DeclarationParser {};

        let properties = DeclarationListParser::new(input, decl_parser)
            .filter_map(|property| property.ok())
            .collect::<Vec<_>>();

        Ok(StyleRule { id: Rule::null(), selectors, properties })
    }
}

impl<'i> cssparser::AtRuleParser<'i> for RuleParser {
    type PreludeBlock = ();
    type PreludeNoBlock = ();
    type AtRule = StyleRule;
    type Error = CustomParseError;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<AtRuleType<Self::PreludeNoBlock, Self::PreludeBlock>, ParseError<'i, Self::Error>>
    {
        match &*name {
            "keyframes" => {
                while let Ok(t) = input.next() {
                    match t {
                        Token::Ident(_animation_name) => {}

                        t => {
                            let basic_error = BasicParseError {
                                kind: BasicParseErrorKind::UnexpectedToken(t.clone()),
                                location: input.current_source_location(),
                            };
                            return Err(basic_error.into());
                        }
                    }
                }
            }

            _ => {
                let token = input.next()?.to_owned();
                return Err(input.new_basic_unexpected_token_error(token).into());
            }
        }

        Ok(AtRuleType::WithBlock(()))
    }

    // TODO
    /*
    fn parse_block<'t>(
        &mut self,
        prelude: Self::PreludeBlock,
        location: SourceLocation,
        input: &mut Parser<'i, 't>
    ) -> Result<Self::AtRule, ParseError<'i, Self::Error>> {
        let rule_parser = RuleParser::new();
        let rule_list_parser = RuleListParser::new_for_nested_rule(input, rule_parser).collect::<Vec<_>>();

        // Keyframe rules
        for rule in &rule_list_parser {

        }
    }
    */
}

fn parse_selectors<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<Vec<Selector>, ParseError<'i, CustomParseError>> {
    let mut selectors: Vec<Selector> = Vec::new();

    let mut selector = Selector::default();

    let mut _first_token_in_selector = true;
    let mut whitespace = false;
    while let Ok(t) = input.next_including_whitespace() {
        match t {
            // Element
            Token::Ident(ref element_name) => {
                if whitespace {
                    selector.relation = SelectorRelation::Ancestor;
                    selectors.push(selector);
                    selector = Selector::default();
                    selector.set_element(element_name);
                } else {
                    selector.set_element(element_name);
                }

                whitespace = false;
            }

            Token::Delim('>') => {
                //let mut old_selector = Selector::from(&input.expect_ident()?.to_string());
                //mem::swap(&mut old_selector, &mut selector);
                selector.relation = SelectorRelation::Parent;
                selectors.push(selector);
                //selector = Selector::from(&input.expect_ident()?.to_string());
                selector = Selector::default();
                whitespace = false;
                // if let Some(selec) = selectors.last_mut() {
                //     selec.relation = Relation::Parent;
                // }
                //selector.relation = Some(Box::new(SelectorRelation::Parent(old_selector)));
            }

            // Id
            Token::IDHash(ref id_name) => {
                selector.set_id(id_name);
                whitespace = false;
            }

            // Any element
            Token::Delim('*') => {
                if whitespace {
                    selector.relation = SelectorRelation::Ancestor;
                    selectors.push(selector);
                    selector = Selector::default();
                    selector.asterisk = true;
                } else {
                    selector.asterisk = true;
                }

                whitespace = false;
            }

            // Class
            Token::Delim('.') => {
                if whitespace {
                    selector.relation = SelectorRelation::Ancestor;
                    selectors.push(selector);
                    selector = Selector::default();
                    selector.classes.insert(input.expect_ident()?.to_owned().to_string());
                } else {
                    selector.classes.insert(input.expect_ident()?.to_owned().to_string());
                }

                whitespace = false;
            }

            Token::WhiteSpace(_ws) => {
                // Parent relations can (and in almost every style guide, do) have whitespace
                // surrounding the `>`. In those cases we should treat this as if there were no
                // whitespace since the rest of this parser uses whitespace strictly to indicate
                // regular nesting. Bit of a hack, but I didn't dare touching the rest of the
                // parser.
                whitespace = !matches!(
                    selectors.last(),
                    Some(Selector { relation: SelectorRelation::Parent, .. })
                );
            }

            // Pseudo-class
            Token::Colon => {
                let pseudo_class_str = input.expect_ident()?.to_owned();

                match pseudo_class_str.as_ref() {
                    "hover" => selector.pseudo_classes.insert(PseudoClass::HOVER),
                    "over" => selector.pseudo_classes.insert(PseudoClass::OVER),
                    "active" => selector.pseudo_classes.insert(PseudoClass::ACTIVE),
                    "focus" => selector.pseudo_classes.insert(PseudoClass::FOCUS),
                    "disabled" => selector.pseudo_classes.insert(PseudoClass::DISABLED),
                    "checked" => selector.pseudo_classes.insert(PseudoClass::CHECKED),
                    "selected" => selector.pseudo_classes.insert(PseudoClass::SELECTED),
                    "custom" => selector.pseudo_classes.insert(PseudoClass::CUSTOM),
                    "focus-within" => selector.pseudo_classes.insert(PseudoClass::FOCUS_WITHIN),
                    "focus-visible" => selector.pseudo_classes.insert(PseudoClass::FOCUS_VISIBLE),
                    "root" => selector.pseudo_classes.insert(PseudoClass::ROOT),

                    _ => {
                        let parse_error = ParseError {
                            kind: ParseErrorKind::Custom(
                                CustomParseError::UnrecognisedPseudoclass(
                                    pseudo_class_str.to_string(),
                                ),
                            ),
                            location: input.current_source_location(),
                        };

                        return Err(parse_error);
                    }
                }
            }

            // This selector is done, on to the next one
            Token::Comma => {
                selectors.push(selector);
                selector = Selector::default();
                _first_token_in_selector = true;
                continue; // need to continue to avoid `first_token_in_selector` being set to false
            }

            t => {
                let basic_error = BasicParseErrorKind::UnexpectedToken(t.to_owned());
                let parse_error = ParseError {
                    kind: ParseErrorKind::Basic(basic_error),
                    location: SourceLocation { line: 0, column: 0 },
                };
                return Err(parse_error);
            }
        }

        _first_token_in_selector = false;
    }

    selectors.push(selector);

    // for selec in selectors.iter() {
    //     println!("{:?}", selec);
    // }

    // if selectors.iter().any(|sel| sel.relation.is_some()) {
    //     eprintln!("WARNING: Complex selector relations not implemented");
    // }

    Ok(selectors)
}

// fn parse_selector<'i,'t>(input: &mut Parser<'i,'t>) -> Result<Selector, ParseError<'i, CustomParseError>> {
//     let mut selector = Selector::default();

//     let token = input.next();

//     match token {
//         Token::Ident(ref element_name) => {
//             selector.set_element(&element_name.to_string());

//         }
//     }
// }

// struct RGBDeclaration;

// impl<'i> cssparser::DeclarationParser<'i> for RGBDeclaration {
//     type Declaration = Color;
//     type Error = CustomParseError;

//     fn parse_block
// }

struct DeclarationParser;

impl<'i> cssparser::DeclarationParser<'i> for DeclarationParser {
    type Declaration = Property;
    type Error = CustomParseError;

    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Declaration, ParseError<'i, Self::Error>> {
        Ok(match &*name {
            // Colors
            "background-color" => Property::BackgroundColor(parse_color(input)?),
            "color" => Property::FontColor(parse_color(input)?),
            "background-image" => Property::BackgroundImage(parse_string(input)?),

            // Position
            "position" | "position-type" => Property::PositionType(parse_position_type(input)?),

            "left" => Property::Left(parse_units(input)?),
            "right" => Property::Right(parse_units(input)?),
            "top" => Property::Top(parse_units(input)?),
            "bottom" => Property::Bottom(parse_units(input)?),
            "space" => Property::Space(parse_units(input)?),

            "min-left" => Property::MinLeft(parse_units(input)?),
            "max-left" => Property::MaxLeft(parse_units(input)?),
            "min-right" => Property::MinRight(parse_units(input)?),
            "max-right" => Property::MaxRight(parse_units(input)?),
            "min-top" => Property::MinTop(parse_units(input)?),
            "max-top" => Property::MaxTop(parse_units(input)?),
            "min-bottom" => Property::MinBottom(parse_units(input)?),
            "max-bottom" => Property::MaxBottom(parse_units(input)?),

            "layout-type" => Property::LayoutType(parse_layout_type(input)?),

            // Size
            "width" => Property::Width(parse_units(input)?),
            "height" => Property::Height(parse_units(input)?),

            // Size Constraints
            "min-width" => Property::MinWidth(parse_units(input)?),
            "min-height" => Property::MinHeight(parse_units(input)?),
            "max-width" => Property::MaxWidth(parse_units(input)?),
            "max-height" => Property::MaxHeight(parse_units(input)?),

            "child-space" => Property::ChildSpace(parse_units(input)?),
            "child-left" => Property::ChildLeft(parse_units(input)?),
            "child-right" => Property::ChildRight(parse_units(input)?),
            "child-top" => Property::ChildTop(parse_units(input)?),
            "child-bottom" => Property::ChildBottom(parse_units(input)?),
            "row-between" => Property::RowBetween(parse_units(input)?),
            "col-between" => Property::ColBetween(parse_units(input)?),
            "font-size" => Property::FontSize(parse_font_size(input)?),
            "font-family" => Property::FontFamily(input.parse_comma_separated(parse_font_family)?),
            "font-weight" => Property::FontWeight(parse_font_weight(input)?),
            "font-style" => Property::FontStyle(parse_font_style(input)?),
            "text-wrap" => Property::TextWrap(parse_bool(input)?),
            "selection-color" => Property::SelectionColor(parse_color(input)?),
            "caret-color" => Property::CaretColor(parse_color(input)?),

            // Border
            "border-width" => Property::BorderWidth(parse_units(input)?),
            "border-color" => Property::BorderColor(parse_color(input)?),
            // TODO - Support array for specifying each corner
            "border-radius" => Property::BorderRadius(parse_units(input)?),
            "border-top-left-radius" => Property::BorderTopLeftRadius(parse_units(input)?),
            "border-top-right-radius" => Property::BorderTopRightRadius(parse_units(input)?),
            "border-bottom-left-radius" => Property::BorderBottomLeftRadius(parse_units(input)?),
            "border-bottom-right-radius" => Property::BorderBottomRightRadius(parse_units(input)?),

            "border-corner-shape" => Property::BorderCornerShape(parse_border_corner_shape(input)?),
            "border-top-left-shape" => {
                Property::BorderTopLeftShape(parse_border_corner_shape(input)?)
            }
            "border-top-right-shape" => {
                Property::BorderTopRightShape(parse_border_corner_shape(input)?)
            }
            "border-bottom-left-shape" => {
                Property::BorderBottomLeftShape(parse_border_corner_shape(input)?)
            }
            "border-bottom-right-shape" => {
                Property::BorderBottomRightShape(parse_border_corner_shape(input)?)
            }

            "outline-width" => Property::OutlineWidth(parse_units(input)?),

            "outline-color" => Property::OutlineColor(parse_color(input)?),

            "outline-offset" => Property::OutlineOffset(parse_units(input)?),

            "opacity" => Property::Opacity(parse_length_or_percentage(input)?),

            "display" => Property::Display(parse_display(input)?),
            "visibility" => Property::Visibility(parse_visibility(input)?),

            "overflow" => Property::Overflow(parse_overflow(input)?),

            "outer-shadow" => Property::OuterShadow(parse_box_shadow(input)?),
            "outer-shadow-h-offset" => Property::OuterShadowHOffset(parse_units(input)?),
            "outer-shadow-v-offset" => Property::OuterShadowVOffset(parse_units(input)?),
            "outer-shadow-blur" => Property::OuterShadowBlur(parse_units(input)?),
            "outer-shadow-color" => Property::OuterShadowColor(parse_color(input)?),

            "inner-shadow" => Property::InnerShadow(parse_box_shadow(input)?),
            "inner-shadow-h-offset" => Property::InnerShadowHOffset(parse_units(input)?),
            "inner-shadow-v-offset" => Property::InnerShadowVOffset(parse_units(input)?),
            "inner-shadow-blur" => Property::InnerShadowBlur(parse_units(input)?),
            "inner-shadow-color" => Property::InnerShadowColor(parse_color(input)?),

            "transition" => Property::Transition(input.parse_comma_separated(parse_transition2)?),

            "z-index" => Property::ZIndex(parse_z_index(input)?),

            "cursor" => Property::Cursor(parse_cursor(input)?),

            ident => Property::Unknown(ident.to_owned(), parse_unknown(input)?),
            // _ => {
            //     let basic_error = BasicParseError {
            //         kind: BasicParseErrorKind::UnexpectedToken(input.next()?.to_owned()),
            //         location: SourceLocation { line: 0, column: 0 },
            //     };
            //     return Err(basic_error.into());
            // }
        })
    }
}

impl<'i> cssparser::AtRuleParser<'i> for DeclarationParser {
    type PreludeNoBlock = ();
    type PreludeBlock = ();
    type AtRule = Property;
    type Error = CustomParseError;
}

fn css_color(name: &str) -> Option<Color> {
    Some(match name {
        "transparent" => Color::from(name),

        "black" => Color::from("#000000"),
        "silver" => Color::from("#C0C0C0"),
        "gray" | "grey" => Color::from("#808080"),
        "white" => Color::from("#FFFFFF"),
        "maroon" => Color::from("#800000"),
        "red" => Color::from("#FF0000"),
        "purple" => Color::from("#800080"),
        "fuchsia" => Color::from("#FF00FF"),
        "green" => Color::from("#008000"),
        "lime" => Color::from("#00FF00"),
        "olive" => Color::from("#808000"),
        "yellow" => Color::from("#FFFF00"),
        "navy" => Color::from("#000080"),
        "blue" => Color::from("#0000FF"),
        "teal" => Color::from("#008080"),
        "aqua" => Color::from("#00FFFF"),
        _ => return None,
    })
}

fn css_string(name: &str) -> Option<String> {
    Some(String::from(name))
}

fn parse_unknown<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<PropType, ParseError<'i, CustomParseError>> {
    Ok(match input.next()? {
        Token::QuotedString(s) => match css_string(s) {
            Some(string) => PropType::String(string),
            None => {
                return Err(CustomParseError::InvalidStringName(s.to_owned().to_string()).into())
            }
        },

        Token::Number { value: x, .. } => PropType::Units(Units::Pixels(*x)),
        Token::Percentage { unit_value: x, .. } => PropType::Units(Units::Percentage(*x)),

        Token::Dimension { has_sign: _, value: v, int_value: _, unit: u } if u == &"px" => {
            PropType::Units(Units::Pixels(*v))
        }

        Token::Dimension { has_sign: _, value: v, int_value: _, unit: u } if u == &"s" => {
            PropType::Units(Units::Stretch(*v))
        }

        Token::Ident(name) if name == &"auto" => PropType::Units(Units::Auto),

        t => {
            let basic_error = BasicParseErrorKind::UnexpectedToken(t.to_owned());
            let parse_error = ParseError {
                kind: ParseErrorKind::Basic(basic_error),
                location: SourceLocation { line: 0, column: 0 },
            };
            return Err(parse_error);
        }
    })
}

fn parse_string<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<String, ParseError<'i, CustomParseError>> {
    Ok(match input.next()? {
        Token::QuotedString(s) => match css_string(s) {
            Some(string) => string,
            None => {
                return Err(CustomParseError::InvalidStringName(s.to_owned().to_string()).into())
            }
        },

        t => {
            let basic_error = BasicParseErrorKind::UnexpectedToken(t.to_owned());
            let parse_error = ParseError {
                kind: ParseErrorKind::Basic(basic_error),
                location: SourceLocation { line: 0, column: 0 },
            };
            return Err(parse_error);
        }
    })
}

// fn parse_basic_color<'i, 't>(
//     input: &mut Parser<'i, 't>,
// ) -> Result<Color, ParseError<'i, CustomParseError>> {
//     Ok(match input.next()? {
//         Token::Ident(s) => match css_color(&s) {
//             Some(color) => color,
//             None => {
//                 return Err(
//                     CustomParseError::UnrecognisedColorName(s.to_owned().to_string()).into()
//                 );
//             }
//         },

//         Token::IDHash(hash) | Token::Hash(hash) => Color::from(hash.to_owned().to_string()),

//         t => {
//             let basic_error = BasicParseErrorKind::UnexpectedToken(t.to_owned());
//             let parse_error = ParseError {
//                 kind: ParseErrorKind::Basic(basic_error),
//                 location: SourceLocation { line: 0, column: 0 },
//             };
//             return Err(parse_error);
//         }
//     })
// }

fn parse_length_or_percentage<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<f32, ParseError<'i, CustomParseError>> {
    Ok(match input.next()? {
        Token::Number { value: x, .. } => *x,
        Token::Percentage { unit_value: x, .. } => *x,

        Token::Dimension { value: x, .. } => *x,
        t => {
            let basic_error = BasicParseError {
                kind: BasicParseErrorKind::UnexpectedToken(t.to_owned()),
                location: SourceLocation { line: 0, column: 0 },
            };
            return Err(basic_error.into());
        }
    })
}

fn parse_z_index<'i>(input: &mut Parser<'i, '_>) -> Result<i32, ParseError<'i, CustomParseError>> {
    Ok(match input.next()? {
        Token::Number { value: x, .. } => *x as i32,
        t => {
            let basic_error = BasicParseError {
                kind: BasicParseErrorKind::UnexpectedToken(t.to_owned()),
                location: SourceLocation { line: 0, column: 0 },
            };
            return Err(basic_error.into());
        }
    })
}

//TODO
// fn parse_transition<'i, 't>(
//     input: &mut Parser<'i, 't>,
//     mut transition: Transition,
// ) -> Result<Transition, ParseError<'i, CustomParseError>> {
//     //let transition = Transition::default();

//     Ok(match input.next()? {
//         Token::Ident(s) => {
//             println!("Transition: {}", s);
//             transition.property = s.to_string();

//             match input.next()? {
//                 Token::Number { value: x, .. } => {
//                     println!("With duration: {}", x);
//                 }

//                 t => {
//                     let basic_error = BasicParseError {
//                         kind: BasicParseErrorKind::UnexpectedToken(t.to_owned()),
//                         location: SourceLocation { line: 0, column: 0 },
//                     };
//                     return Err(basic_error.into());
//                 }
//             }

//             transition
//         }

//         t => {
//             let basic_error = BasicParseError {
//                 kind: BasicParseErrorKind::UnexpectedToken(t.to_owned()),
//                 location: SourceLocation { line: 0, column: 0 },
//             };
//             return Err(basic_error.into());
//         }
//     })
// }

//TODO
fn parse_box_shadow<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<BoxShadow, ParseError<'i, CustomParseError>> {
    let horizontal_offset = parse_length2(input.next()?)?;
    let vertical_offset = parse_length2(input.next()?)?;

    let mut box_shadow = BoxShadow { horizontal_offset, vertical_offset, ..Default::default() };

    let next_token = input.next()?;
    let color = if let Ok(units) = parse_length2(next_token) {
        box_shadow.blur_radius = units;

        input.next()?
    } else {
        next_token
    };
    if let Ok(color) = parse_color2(color) {
        box_shadow.color = color
    }

    Ok(box_shadow)

    // match token {
    //     Token::Number { value: x, .. } => {
    //         box_shadow.horizontal_offset = Units::Pixels(*x);

    //         match input.next()? {
    //             Token::Number { value: x, .. } => {
    //                 box_shadow.vertical_offset = Units::Pixels(*x);
    //             }

    //             t => {}
    //         }
    //     }

    //     Token::Percentage { unit_value: x, .. } => Units::Percentage(*x as f32),

    //     Token::Dimension { value: x, .. } => Units::Pixels(*x as f32),

    //     t => {
    //         let basic_error = BasicParseError {
    //             kind: BasicParseErrorKind::UnexpectedToken(t.to_owned()),
    //             location: SourceLocation { line: 0, column: 0 },
    //         };
    //         return Err(basic_error.into());
    //     }
    // }

    // Ok(box_shadow)
}

fn parse_length2<'i>(token: &Token<'i>) -> Result<Units, ParseError<'i, CustomParseError>> {
    match token {
        Token::Number { value: x, .. } => Ok(Units::Pixels(*x)),
        Token::Percentage { unit_value: x, .. } => Ok(Units::Percentage(*x)),

        Token::Dimension { value: x, .. } => Ok(Units::Pixels(*x)),
        t => {
            let basic_error = BasicParseError {
                kind: BasicParseErrorKind::UnexpectedToken(t.to_owned()),
                location: SourceLocation { line: 0, column: 0 },
            };
            Err(basic_error.into())
        }
    }
}

fn parse_transition2<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<Transition, ParseError<'i, CustomParseError>> {
    let mut transition = Transition::new();

    Ok(match input.next()? {
        Token::Ident(s) => {
            transition.property = s.to_string();

            match input.next()? {
                Token::Number { value: x, .. } => {
                    transition.duration = *x;

                    match input.next()? {
                        Token::Number { value: x, .. } => {
                            transition.delay = *x;
                        }

                        _t => {
                            // let basic_error = BasicParseError {
                            //     kind: BasicParseErrorKind::UnexpectedToken(t.to_owned()),
                            //     location: SourceLocation { line: 0, column: 0 },
                            // };
                            // return Err(basic_error.into());
                        }
                    }
                }

                t => {
                    let basic_error = BasicParseError {
                        kind: BasicParseErrorKind::UnexpectedToken(t.to_owned()),
                        location: SourceLocation { line: 0, column: 0 },
                    };
                    return Err(basic_error.into());
                }
            }

            transition
        }

        t => {
            let basic_error = BasicParseError {
                kind: BasicParseErrorKind::UnexpectedToken(t.to_owned()),
                location: SourceLocation { line: 0, column: 0 },
            };
            return Err(basic_error.into());
        }
    })
}

fn parse_units<'i>(input: &mut Parser<'i, '_>) -> Result<Units, ParseError<'i, CustomParseError>> {
    Ok(match input.next()? {
        Token::Number { value: x, .. } => Units::Pixels(*x),
        Token::Percentage { unit_value: x, .. } => Units::Percentage(*x * 100.0),

        Token::Dimension { has_sign: _, value: v, int_value: _, unit: u } if u == &"px" => {
            Units::Pixels(*v)
        }

        Token::Dimension { has_sign: _, value: v, int_value: _, unit: u } if u == &"s" => {
            Units::Stretch(*v)
        }

        Token::Ident(name) if name == &"auto" => Units::Auto,

        t => {
            let basic_error = BasicParseError {
                kind: BasicParseErrorKind::UnexpectedToken(t.to_owned()),
                location: SourceLocation { line: 0, column: 0 },
            };
            return Err(basic_error.into());
        }
    })
}

fn parse_position_type<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<PositionType, ParseError<'i, CustomParseError>> {
    let location = input.current_source_location();

    Ok(match input.next()? {
        Token::Ident(name) => match name.as_ref() {
            "self-directed" => PositionType::SelfDirected,
            "parent-directed" => PositionType::ParentDirected,

            _t => {
                return Err(CustomParseError::InvalidStringName(name.to_owned().to_string()).into());
            }
        },

        t => {
            let basic_error = BasicParseError {
                kind: BasicParseErrorKind::UnexpectedToken(t.to_owned()),
                location,
            };
            return Err(basic_error.into());
        }
    })
}

fn parse_cursor<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<CursorIcon, ParseError<'i, CustomParseError>> {
    let location = input.current_source_location();

    Ok(match input.next()? {
        Token::Ident(name) => match name.as_ref() {
            "default" => CursorIcon::Default,
            "crosshair" => CursorIcon::Crosshair,
            "hand" => CursorIcon::Hand,
            "arrow" => CursorIcon::Arrow,
            "move" => CursorIcon::Move,
            "text" => CursorIcon::Text,
            "wait" => CursorIcon::Wait,
            "help" => CursorIcon::Help,
            "progress" => CursorIcon::Progress,
            "not-allowed" => CursorIcon::NotAllowed,
            "context-menu" => CursorIcon::ContextMenu,
            "cell" => CursorIcon::Cell,
            "vertical-text" => CursorIcon::VerticalText,
            "alias" => CursorIcon::Alias,
            "copy" => CursorIcon::Copy,
            "no-drop" => CursorIcon::NoDrop,
            "grab" => CursorIcon::Grab,
            "grabbing" => CursorIcon::Grabbing,
            "all-scroll" => CursorIcon::AllScroll,
            "zoom-in" => CursorIcon::ZoomIn,
            "zoom-out" => CursorIcon::ZoomOut,
            "e-resize" => CursorIcon::EResize,
            "n-resize" => CursorIcon::NResize,
            "ne-resize" => CursorIcon::NeResize,
            "nw-resize" => CursorIcon::NwResize,
            "s-resize" => CursorIcon::SResize,
            "se-resize" => CursorIcon::SeResize,
            "sw-resize" => CursorIcon::SwResize,
            "w-resize" => CursorIcon::WResize,
            "ew-resize" => CursorIcon::EwResize,
            "ns-resize" => CursorIcon::NsResize,
            "nesw-resize" => CursorIcon::NeswResize,
            "nwse-resize" => CursorIcon::NwseResize,
            "col-resize" => CursorIcon::ColResize,
            "row-resize" => CursorIcon::RowResize,
            "none" => CursorIcon::None,

            _t => {
                return Err(CustomParseError::InvalidStringName(name.to_owned().to_string()).into());
            }
        },

        t => {
            let basic_error = BasicParseError {
                kind: BasicParseErrorKind::UnexpectedToken(t.to_owned()),
                location,
            };
            return Err(basic_error.into());
        }
    })
}

fn parse_display<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<Display, ParseError<'i, CustomParseError>> {
    let location = input.current_source_location();

    Ok(match input.next()? {
        Token::Ident(name) => match name.as_ref() {
            "none" => Display::None,
            "flex" => Display::Flex,

            _ => {
                return Err(CustomParseError::InvalidStringName(name.to_owned().to_string()).into());
            }
        },

        t => {
            let basic_error = BasicParseError {
                kind: BasicParseErrorKind::UnexpectedToken(t.to_owned()),
                location,
            };
            return Err(basic_error.into());
        }
    })
}

fn parse_bool<'i>(input: &mut Parser<'i, '_>) -> Result<bool, ParseError<'i, CustomParseError>> {
    let location = input.current_source_location();

    Ok(match input.next()? {
        Token::Ident(name) => match name.as_ref() {
            "on" => true,
            "off" => false,
            "true" => true,
            "false" => false,
            "yes" => true,
            "no" => false,
            "maybe" => ((parse_bool as *const u8 as usize >> 16) & 1) != 0,

            _ => {
                return Err(CustomParseError::InvalidStringName(name.to_owned().to_string()).into());
            }
        },

        t => {
            let basic_error = BasicParseError {
                kind: BasicParseErrorKind::UnexpectedToken(t.to_owned()),
                location,
            };
            return Err(basic_error.into());
        }
    })
}

fn parse_visibility<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<Visibility, ParseError<'i, CustomParseError>> {
    let location = input.current_source_location();

    Ok(match input.next()? {
        Token::Ident(name) => match name.as_ref() {
            "visible" => Visibility::Visible,
            "hidden" => Visibility::Invisible,

            _ => {
                return Err(CustomParseError::InvalidStringName(name.to_owned().to_string()).into());
            }
        },

        t => {
            let basic_error = BasicParseError {
                kind: BasicParseErrorKind::UnexpectedToken(t.to_owned()),
                location,
            };
            return Err(basic_error.into());
        }
    })
}

fn parse_overflow<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<Overflow, ParseError<'i, CustomParseError>> {
    let location = input.current_source_location();

    Ok(match input.next()? {
        Token::Ident(name) => match name.as_ref() {
            "visible" => Overflow::Visible,
            "hidden" => Overflow::Hidden,

            _ => {
                return Err(CustomParseError::InvalidStringName(name.to_owned().to_string()).into());
            }
        },

        t => {
            let basic_error = BasicParseError {
                kind: BasicParseErrorKind::UnexpectedToken(t.to_owned()),
                location,
            };
            return Err(basic_error.into());
        }
    })
}

fn parse_border_corner_shape<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<BorderCornerShape, ParseError<'i, CustomParseError>> {
    let location = input.current_source_location();

    Ok(match input.next()? {
        Token::Ident(name) => match name.as_ref() {
            "round" => BorderCornerShape::Round,
            "bevel" => BorderCornerShape::Bevel,

            _ => {
                return Err(CustomParseError::InvalidStringName(name.to_owned().to_string()).into());
            }
        },

        t => {
            let basic_error = BasicParseError {
                kind: BasicParseErrorKind::UnexpectedToken(t.to_owned()),
                location,
            };
            return Err(basic_error.into());
        }
    })
}

fn parse_layout_type<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<LayoutType, ParseError<'i, CustomParseError>> {
    let location = input.current_source_location();

    Ok(match input.next()? {
        Token::Ident(name) => match name.as_ref() {
            "row" => LayoutType::Row,
            "column" => LayoutType::Column,
            "grid" => LayoutType::Grid,

            _ => {
                return Err(CustomParseError::InvalidStringName(name.to_owned().to_string()).into());
            }
        },

        t => {
            let basic_error = BasicParseError {
                kind: BasicParseErrorKind::UnexpectedToken(t.to_owned()),
                location,
            };
            return Err(basic_error.into());
        }
    })
}

fn parse_color<'i>(input: &mut Parser<'i, '_>) -> Result<Color, ParseError<'i, CustomParseError>> {
    let location = input.current_source_location();

    Ok(match input.next()? {
        Token::Ident(name) => {
            // if input.try_parse(|input| input.expect_ident_matching("rgb")).is_ok() {
            //     if input.expect_parenthesis_block().is_ok() {
            //         input.parse_nested_block(parse: F)
            //     }
            // }

            match css_color(name) {
                Some(color) => color,
                None => {
                    return Err(CustomParseError::UnrecognisedColorName(
                        name.to_owned().to_string(),
                    )
                    .into());
                }
            }
        }

        Token::IDHash(hash) | Token::Hash(hash) => Color::from(hash.to_owned().to_string()),

        t => {
            let basic_error = BasicParseError {
                kind: BasicParseErrorKind::UnexpectedToken(t.to_owned()),
                location,
            };
            return Err(basic_error.into());
        }
    })
}

fn parse_color2<'i>(token: &Token<'i>) -> Result<Color, ParseError<'i, CustomParseError>> {
    match token {
        Token::Ident(name) => {
            // if input.try_parse(|input| input.expect_ident_matching("rgb")).is_ok() {
            //     if input.expect_parenthesis_block().is_ok() {
            //         input.parse_nested_block(parse: F)
            //     }
            // }

            match css_color(name) {
                Some(color) => Ok(color),
                None => {
                    Err(CustomParseError::UnrecognisedColorName(name.to_owned().to_string()).into())
                }
            }
        }

        Token::IDHash(hash) | Token::Hash(hash) => Ok(Color::from(hash.to_owned().to_string())),

        t => {
            let basic_error = BasicParseError {
                kind: BasicParseErrorKind::UnexpectedToken(t.to_owned()),
                location: SourceLocation { line: 0, column: 0 },
            };
            Err(basic_error.into())
        }
    }
}

fn parse_font_weight<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<Weight, ParseError<'i, CustomParseError>> {
    let location = input.current_source_location();
    Ok(match input.next()? {
        Token::Number { value, has_sign: false, .. } => {
            if *value <= 0.0 || *value > 1000.0 {
                return Err(BasicParseError {
                    kind: BasicParseErrorKind::QualifiedRuleInvalid,
                    location,
                }
                .into());
            }
            Weight(value.trunc() as u16)
        }
        Token::Ident(name) if name.as_ref() == "normal" => Weight::NORMAL,
        Token::Ident(name) if name.as_ref() == "bold" => Weight::BOLD,
        t => {
            return Err(BasicParseError {
                kind: BasicParseErrorKind::UnexpectedToken(t.to_owned()),
                location,
            }
            .into())
        }
    })
}

fn parse_font_style<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<FontStyle, ParseError<'i, CustomParseError>> {
    let location = input.current_source_location();
    Ok(match input.next()? {
        Token::Ident(name) if name.as_ref() == "normal" => FontStyle::Normal,
        Token::Ident(name) if name.as_ref() == "italic" => FontStyle::Italic,
        Token::Ident(name) if name.as_ref() == "oblique" => FontStyle::Oblique,
        t => {
            return Err(BasicParseError {
                kind: BasicParseErrorKind::UnexpectedToken(t.to_owned()),
                location,
            }
            .into())
        }
    })
}

pub(crate) fn parse_font_family<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<FamilyOwned, ParseError<'i, CustomParseError>> {
    let location = input.current_source_location();
    match input.next()? {
        Token::Ident(name) | Token::QuotedString(name) => Ok(match name.as_ref() {
            "serif" => FamilyOwned::Serif,
            "sans-serif" => FamilyOwned::SansSerif,
            "monospace" => FamilyOwned::Monospace,
            "cursive" => FamilyOwned::Cursive,
            "fantasy" => FamilyOwned::Fantasy,
            name => FamilyOwned::Name(name.to_owned()),
        }),
        t => Err(BasicParseError {
            kind: BasicParseErrorKind::UnexpectedToken(t.to_owned()),
            location,
        }
        .into()),
    }
}

fn parse_font_size<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<f32, ParseError<'i, CustomParseError>> {
    let location = input.current_source_location();

    Ok(match input.next()? {
        Token::Ident(name) => match name.as_ref() {
            "medium" => 14.0,
            "xx-small" => 8.0,
            "x-small" => 10.0,
            "small" => 12.0,
            "large" => 16.0,
            "x-large" => 18.0,
            "xx-large" => 20.0,

            _ => {
                return Err(CustomParseError::InvalidStringName(name.to_owned().to_string()).into());
            }
        },

        Token::Number { value: x, .. } => *x,
        Token::Percentage { unit_value: x, .. } => *x,

        Token::Dimension { value: x, .. } => *x,

        t => {
            let basic_error = BasicParseError {
                kind: BasicParseErrorKind::UnexpectedToken(t.to_owned()),
                location,
            };
            return Err(basic_error.into());
        }
    })
}

pub(crate) fn _parse(s: &str) -> Vec<StyleRule> {
    let mut input = ParserInput::new(s);
    let mut parser = Parser::new(&mut input);
    let rule_parser = RuleParser::new();

    let rules = {
        let rule_list_parser =
            cssparser::RuleListParser::new_for_stylesheet(&mut parser, rule_parser);
        rule_list_parser.collect::<Vec<_>>()
    };

    rules.into_iter().filter_map(|rule| rule.ok()).collect()
}
