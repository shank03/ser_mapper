/// Implement DTO serialzation wrapper for DBO/Model
///
/// Instead of mapping DBO/Model to DTO then serializing DTO,
/// this macro will help you create the skeleton of DTO and assign
/// and/or map the values you want each field of DTO to have
/// from DBO/Model.
///
/// Let's say your DTO is `UserResponse` and DBO/Model is `User`.
/// This macro will generate the following structs for serializing
/// (not mapping) `User` into `UserResponse`:
/// - `struct UserResponse { ... };`
/// - `struct _UserResponse(pub User);`
/// - `struct _UserResponseRef<'a>(pub &'a User);`
/// - `struct _UserResponseRefOption<'a>(pub &'a Option<User>);`
/// - `struct _UserResponseOptionRef<'a>(pub Option<&'a User>);`
/// - `struct _UserResponseVec(pub Vec<User>);`
/// - `struct _UserResponseVecRef<'a>(pub Vec<&'a User>);`
///
/// and all the structs starting with a `_` will implement [`serde::Serialize`].
///
/// You can then just wrap the DBO/Model into DTO as `Dto(Dbo/model)`
/// and [`serde`] will serialize DBO/Model to DTO according to mapping.
///
/// Example:
/// ```
/// use ser_mapper::impl_dto;
///
/// #[derive(Debug)]
/// struct TableId {
///     table: String,
///     id: String,
/// }
/// impl TableId {
///     fn get_id(t: &TableId) -> &str {
///         &t.id
///     }
/// }
///
/// #[derive(Debug)]
/// struct Age(u8);
///
/// #[derive(Debug)]
/// struct DboModel {
///     id: TableId,
///     full_name: String,
///     email: String,
///     age: Age,
/// }
/// impl DboModel {
///     fn get_first_name(name: &str) -> &str {
///         name.split(" ").nth(0).unwrap()
///     }
///     fn get_last_name(name: &str) -> &str {
///         name.split(" ").nth(1).unwrap()
///     }
/// }
///
/// // Creates the following wrappers:
/// // - `struct Dto { id: String, first_name: String, last_name: String, email: String, age: u8 };`
/// // - `struct _Dto(pub DboModel);`
/// // - `struct _DtoRef<'a>(pub &'a DboModel);`
/// // - `struct _DtoRefOption<'a>(pub &'a Option<DboModel>);`
/// // - `struct _DtoOptionRef<'a>(pub Option<&'a DboModel>);`
/// // - `struct _DtoVec(pub Vec<DboModel>);`
/// // - `struct _DtoVecRef<'a>(pub Vec<&'a DboModel>);`
/// // and all of these implement [`serde::Serialize`]
/// impl_dto!(
///     #[derive(Debug)]
///     struct Dto<DboModel> {
///         user_id: String = id => TableId::get_id,
///         first_name: String = full_name => DboModel::get_first_name,
///         last_name: String = full_name => DboModel::get_last_name,
///         email_id: String = email,
///         age: u8 = age => |a: &Age| a.0,
///     }
/// );
///
/// fn main() {
///     let dbo = DboModel {
///         id: TableId { table: String::from("User"), id: String::from("abcd_123") },
///         full_name: String::from("John Doe"),
///         email: String::from("jd@email.com"),
///         age: Age(69),
///     };
///
///     let dto = _Dto(dbo);
///     assert_eq!(r#"{
///   "user_id": "abcd_123",
///   "first_name": "John",
///   "last_name": "Doe",
///   "email_id": "jd@email.com",
///   "age": 69
/// }"#,
///         serde_json::to_string_pretty(&dto).unwrap()
///     );
/// }
/// ```
#[macro_export]
macro_rules! impl_dto {
    // $(<$($l:lifetime),*>)?
    // $(<$($generic $(= $g_ty)?),*>)?
    (
        $(#[$m:meta])*
        $vis:vis struct $dto:ident<$inner_entity:ty> {
            $(
                $(#[$field_m:meta])*
                $field_vis:vis $field:ident: $field_ty:ty = $($inner_path:ident).+ $(=> $st_expr:expr)?,
            )*
        }
    ) => {
        $(#[$m])*
        $vis struct $dto {
            $(
                $(#[$field_m])*
                $field_vis $field: $field_ty,
            )+
        }

        paste::paste! {
            trait [<$dto Serializer>] {
                fn dto_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer;
            }

            $vis struct [<_ $dto>](pub $inner_entity);
            impl serde::Serialize for [<_ $dto>] {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    self.0.dto_serialize(serializer)
                }
            }

            $vis struct [<_ $dto Ref>]<'a>(pub &'a $inner_entity);
            impl<'a> serde::Serialize for [<_ $dto Ref>]<'a> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    self.0.dto_serialize(serializer)
                }
            }

            $vis struct [<_ $dto RefOption>]<'a>(pub &'a Option<$inner_entity>);
            impl<'a> serde::Serialize for [<_ $dto RefOption>]<'a> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    match self.0 {
                        Some(inner) => [<_ $dto Ref>](inner).serialize(serializer),
                        None => serializer.serialize_none(),
                    }
                }
            }

            $vis struct [<_ $dto OptionRef>]<'a>(pub Option<&'a $inner_entity>);
            impl<'a> serde::Serialize for [<_ $dto OptionRef>]<'a> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    match self.0 {
                        Some(inner) => [<_ $dto Ref>](inner).serialize(serializer),
                        None => serializer.serialize_none(),
                    }
                }
            }

            $vis struct [<_ $dto VecRef>]<'a>(pub &'a Vec<$inner_entity>);
            impl<'a> serde::Serialize for [<_ $dto VecRef>]<'a> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    use serde::ser::SerializeSeq;

                    let mut state = serializer.serialize_seq(Some(self.0.len()))?;
                    for inner in self.0.iter() {
                        let item = [<_ $dto Ref>](inner);
                        state.serialize_element(&item)?;
                    }
                    state.end()
                }
            }

            $vis struct [<_ $dto Vec>](pub Vec<$inner_entity>);
            impl serde::Serialize for [<_ $dto Vec>] {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    [<_ $dto VecRef>](&self.0).serialize(serializer)
                }
            }

            impl [<$dto Serializer>] for $inner_entity {
                fn dto_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    use serde::ser::SerializeStruct;

                    let mut state = serializer.serialize_struct(stringify!($dto), impl_dto!(@count $($field),+))?;
                    $(
                        {
                            let value = &self.$($inner_path).+;
                            let value = $($st_expr)?(value);
                            state.serialize_field(stringify!($field), &value)?;
                        }
                    )*
                    state.end()
                }
            }
        }
    };
    (@count $t1:tt, $($t:tt),+) => { 1 + impl_dto!(@count $($t),+) };
    (@count $t:tt) => { 1 };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapper() {
        #[derive(Debug)]
        struct Age(u8);

        #[derive(Debug)]
        struct Dbo {
            id: String,
            full_name: String,
            age: Age,
        }

        fn get_first_name(a: &str) -> &str {
            a.split(" ").nth(0).unwrap()
        }
        fn get_last_name(a: &str) -> &str {
            a.split(" ").nth(1).unwrap()
        }

        // Define mapped DTO
        impl_dto!(
            #[derive(Debug)]
            struct Dto<Dbo> {
                user_id: String = id,
                first_name: String = full_name => get_first_name,
                last_name: String = full_name => get_last_name,
                age: u8 = age => |a: &Age| a.0,
            }
        );

        let dbo = Dbo {
            id: String::from("abcdid_123"),
            full_name: String::from("Hello world!"),
            age: Age(69),
        };

        let dto = _Dto(dbo);
        assert_eq!(
            r#"{"user_id":"abcdid_123","first_name":"Hello","last_name":"world!","age":69}"#,
            serde_json::to_string(&dto).unwrap()
        );
    }
}
