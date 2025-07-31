use super::{CoreType, FileContents, RepackError, RepackErrorKind, Token};

// #[derive(Debug)]
// pub enum QueryType {
//     Select,
//     Update,
//     Delete,
// }
//
// #[derive(Debug)]
// pub enum QueryCommands {
//     List,
//     Filter,
//     Sortd,
//     Sorta,
//     Limit,
// }
// impl QueryCommands {
//     pub fn from_string(val: &str) -> Option<Self> {
//         Some(match val {
//             "list" => Self::List,
//             "filter" => Self::Filter,
//             "sortd" => Self::Sortd,
//             "sorta" => Self::Sorta,
//             "limit" => Self::Limit,
//             _ => return None,
//         })
//     }
// }
//
// #[derive(Debug)]
// pub struct QueryArg {
//     pub name: String,
//     pub arg_type: CoreType,
// }
//
// #[derive(Debug)]
// pub struct Query {
//     pub name: String,
//     pub args: Vec<QueryArg>,
//     pub commands: Vec<QueryArg>,
// }
// impl Query {
//     pub fn from_contents(
//         obj_name: &str,
//         contents: &mut FileContents,
//     ) -> Result<Query, RepackError> {
//         let name = match contents.take() {
//             Some(Token::Literal(val)) => val,
//             _ => {
//                 return Err(RepackError::global(
//                     RepackErrorKind::QueryInvalidSyntax,
//                     obj_name.to_string(),
//                 ));
//             }
//         };
//         let mut args = Vec::new();
//         // Parse header
//         loop {
//             let Some(next) = contents.take() else {
//                 return Err(RepackError::global(
//                     RepackErrorKind::QueryInvalidSyntax,
//                     obj_name.to_string(),
//                 ));
//             };
//             match next {
//                 Token::Literal(var_name) => {
//                     if !matches!(contents.take(), Some(Token::Colon)) {
//                         return Err(RepackError::global(
//                             RepackErrorKind::QueryInvalidSyntax,
//                             obj_name.to_string(),
//                         ));
//                     }
//                     let Some(Token::Literal(arg_type_str)) = contents.take() else {
//                         return Err(RepackError::global(
//                             RepackErrorKind::QueryInvalidSyntax,
//                             obj_name.to_string(),
//                         ));
//                     };
//                     let Some(arg_type) = CoreType::from_string(&arg_type_str) else {
//                         return Err(RepackError::global(
//                             RepackErrorKind::QueryInvalidSyntax,
//                             obj_name.to_string(),
//                         ));
//                     };
//                     args.push(QueryArg {
//                         name: var_name,
//                         arg_type,
//                     });
//                 }
//                 Token::CloseParen => break,
//                 _ => {}
//             }
//         }
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use crate::syntax::{Object, ObjectType};
//
//     use super::*;
//
//     #[test]
//     fn basic_parse() {
//         let query_string = "User @users #model {
//                 !base
//                 last_login datetime?
//                 name string
//                 email string
//                 user_type UserType
//                 subscription_id string?
//
//                 query SelectFromEmail {
//                     type select
//                     args {
//                         email string
//                         limit int32
//                     }
//                     where {
//                         \"%email = $email\"
//                     }
//                     limit $limit
//                 }
//             }
//         ";
//         let mut contents = FileContents::empty();
//         contents.add_string(&query_string);
//         let obj = Object::read_from_contents(ObjectType::Record, &mut contents);
//         assert_eq!(obj.queries.len(), 1);
//         let query = obj.queries.first().unwrap();
//         assert_eq!(query.name, "SelectFromEmail");
//         dbg!(obj.render_query(query));
//     }
// }
