#[doc(hidden)]
pub use paste;

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
/// - `struct UserResponse { /* mentioned fields in macro */ };`
/// - `struct _UserResponse(pub User);`
/// - `struct _UserResponseRef<'a>(pub &'a User);`
/// - `struct _UserResponseOption(pub Option<User>);`
/// - `struct _UserResponseRefOption<'a>(pub Option<&'a User>);`
/// - `struct _UserResponseOptionRef<'a>(pub &'a Option<User>);`
/// - `struct _UserResponseVec(pub Vec<User>);`
/// - `struct _UserResponseRefVec<'a>(pub Vec<&'a User>);`
/// - `struct _UserResponseVecRef<'a>(pub &'a Vec<User>);`
///
/// and all the structs starting with a `_` will implement [`serde::Serialize`].
///
/// You can then just wrap the DBO/Model into DTO as `_Dto_Kind(Dbo/model)`
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
///
/// // Creates the following wrappers:
/// // - `struct Dto { id: String, first_name: String, last_name: String, email: String, age: u8 };`
/// // - `struct _Dto(pub DboModel);`
/// // - `struct _DtoRef<'a>(pub &'a DboModel);`
/// // - `struct _DtoOption(pub Option<DboModel>);`
/// // - `struct _DtoRefOption<'a>(pub Option<&'a DboModel>);`
/// // - `struct _DtoOptionRef<'a>(pub &'a Option<DboModel>);`
/// // - `struct _DtoVec(pub Vec<DboModel>);`
/// // - `struct _DtoVecRef<'a>(pub &'a Vec<DboModel>);`
/// // - `struct _DtoRefVec<'a>(pub Vec<&'a DboModel>);`
/// // and all of these implement [`serde::Serialize`]
/// impl_dto!(
///     #[derive(Debug)]
///     struct Dto<DboModel> {
///         user_id: String = id => |id: &TableId| -> &str { &id.id },
///         first_name: String = full_name => |n: &String| -> String { n.split(" ").nth(0).unwrap().to_owned() },
///         last_name: String = full_name => |n: &String| -> String { n.split(" ").nth(1).unwrap().to_owned() },
///         email_id: String = email,
///         age: u8 = age => |a: &Age| -> u8 { a.0 },
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
    (
        $(#[$m:meta])*
        $vis:vis struct $dto:ident<$inner_entity:ty> {
            $(
                $(#[$field_m:meta])*
                $field_vis:vis $field:ident: $field_ty:ty = $($inner_path:ident).+ $(=> $fn_expr:expr)?,
            )*
        }
    ) => {

        impl_dto!(@define_dto
            $(#[$m])*
            $vis struct $dto<$inner_entity> {
                $(
                    $(#[$field_m])*
                    $field_vis $field: $field_ty,
                )*
            }
        );

        $crate::paste::paste! {
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
                            let value = $($fn_expr)?(value);
                            state.serialize_field(stringify!($field), &value)?;
                        }
                    )*
                    state.end()
                }
            }
        }
    };
    (
        @define_dto
        $(#[$m:meta])*
        $vis:vis struct $dto:ident<$inner_entity:ty> {
            $(
                $(#[$field_m:meta])*
                $field_vis:vis $field:ident: $field_ty:ty,
            )*
        }
    ) => {
        #[allow(dead_code)]
        $(#[$m])*
        $vis struct $dto {
            $(
                $(#[$field_m])*
                $field_vis $field: $field_ty,
            )+
        }

        $crate::paste::paste! {
            trait [<$dto Serializer>] {
                fn dto_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer;
            }

            $vis struct [<_ $dto>](pub $inner_entity);
            $vis struct [<_ $dto Ref>]<'a>(pub &'a $inner_entity);
            $vis struct [<_ $dto Option>](pub Option<$inner_entity>);
            $vis struct [<_ $dto RefOption>]<'a>(pub Option<&'a $inner_entity>);
            $vis struct [<_ $dto OptionRef>]<'a>(pub &'a Option<$inner_entity>);
            $vis struct [<_ $dto Vec>](pub Vec<$inner_entity>);
            $vis struct [<_ $dto RefVec>]<'a>(pub Vec<&'a $inner_entity>);
            $vis struct [<_ $dto VecRef>]<'a>(pub &'a Vec<$inner_entity>);

            impl serde::Serialize for [<_ $dto>] {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    self.0.dto_serialize(serializer)
                }
            }
            impl<'a> serde::Serialize for [<_ $dto Ref>]<'a> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    self.0.dto_serialize(serializer)
                }
            }
            impl serde::Serialize for [<_ $dto Option>] {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    match &self.0 {
                        Some(inner) => [<_ $dto Ref>](inner).serialize(serializer),
                        None => serializer.serialize_none(),
                    }
                }
            }
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
            impl serde::Serialize for [<_ $dto Vec>] {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    [<_ $dto VecRef>](&self.0).serialize(serializer)
                }
            }
            impl<'a> serde::Serialize for [<_ $dto RefVec>]<'a> {
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
        }
    };
    (@count $t1:tt, $($t:tt),+) => { 1 + impl_dto!(@count $($t),+) };
    (@count $t:tt) => { 1 };
}

#[cfg(test)]
mod tests {
    use super::*;

    impl_dto!(
        @define_dto
        #[derive(Debug)]
        pub struct Id<RecordId> {
            pub id: String,
        }
    );

    impl IdSerializer for RecordId {
        fn dto_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_str(self.key)
        }
    }

    #[derive(Debug)]
    pub struct RecordId {
        #[allow(dead_code)]
        pub table: &'static str,
        pub key: &'static str,
    }

    #[derive(Debug)]
    struct Dbo {
        id: RecordId,
        full_name: String,
        age: u8,
    }
    impl Dbo {
        fn new() -> Self {
            Dbo {
                id: RecordId {
                    table: "user",
                    key: "abdc_123",
                },
                full_name: String::from("Hello world!"),
                age: 69,
            }
        }
    }

    // Define mapped DTO
    impl_dto!(
        #[derive(Debug)]
        struct Dto<Dbo> {
            user_id: String = id => _IdRef,
            first_name: String = full_name => |f_name: &String| -> String { f_name.split(" ").nth(0).unwrap().to_owned() },
            last_name: String = full_name => |n: &String| -> String { n.split(" ").nth(1).unwrap().to_owned() },
            age: u8 = age,
        }
    );

    const EXP_SER: &str =
        r#"{"user_id":"abcd_123","first_name":"Hello","last_name":"world!","age":69}"#;
    const EXP_NULL: &str = "null";

    #[test]
    fn test_ser_mapper() {
        let dbo = Dbo::new();
        let dto = _Dto(dbo);
        assert_eq!(EXP_SER, serde_json::to_string(&dto).unwrap());
    }

    #[test]
    fn test_ser_mapper_ref() {
        let dbo = Dbo::new();
        let dto = _DtoRef(&dbo);
        assert_eq!(EXP_SER, serde_json::to_string(&dto).unwrap());
    }

    #[test]
    fn test_ser_mapper_option() {
        let dbo = Dbo::new();
        let dto_some = _DtoOption(Some(dbo));
        let dto_none = _DtoOption(None);

        assert_eq!(EXP_SER, serde_json::to_string(&dto_some).unwrap());
        assert_eq!(EXP_NULL, serde_json::to_string(&dto_none).unwrap());
    }

    #[test]
    fn test_ser_mapper_option_ref() {
        let dbo = Some(Dbo::new());
        let dto_some = _DtoOptionRef(&dbo);
        let dto_none = _DtoOptionRef(&None);

        assert_eq!(EXP_SER, serde_json::to_string(&dto_some).unwrap());
        assert_eq!(EXP_NULL, serde_json::to_string(&dto_none).unwrap());
    }

    #[test]
    fn test_ser_mapper_ref_option() {
        let dbo = Dbo::new();
        let dto_some = _DtoRefOption(Some(&dbo));
        let dto_none = _DtoRefOption(None);

        assert_eq!(EXP_SER, serde_json::to_string(&dto_some).unwrap());
        assert_eq!(EXP_NULL, serde_json::to_string(&dto_none).unwrap());
    }

    #[test]
    fn test_ser_mapper_vec() {
        let dbo = Dbo::new();
        let dto = _DtoVec(vec![dbo]);

        assert_eq!(format!("[{EXP_SER}]"), serde_json::to_string(&dto).unwrap());
    }

    #[test]
    fn test_ser_mapper_vec_ref() {
        let dbo = vec![Dbo::new()];
        let dto = _DtoVecRef(&dbo);

        assert_eq!(format!("[{EXP_SER}]"), serde_json::to_string(&dto).unwrap());
    }

    #[test]
    fn test_ser_mapper_ref_vec() {
        let dbo = Dbo::new();
        let dto = _DtoRefVec(vec![&dbo]);

        assert_eq!(format!("[{EXP_SER}]"), serde_json::to_string(&dto).unwrap());
    }
}
