# `ser_mapper`

[<img alt="crates.io" src="https://img.shields.io/crates/v/ser_mapper.svg?style=for-the-badge&color=fc8d62&logo=rust" height="22">](https://crates.io/crates/ser_mapper)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-ser_mapper-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="22">](https://docs.rs/ser_mapper)

Serialization Mapper.

Implement mapped DTO serialzation wrapper for DBO/Model without mapping objects.

Instead of mapping DBO/Model to DTO then serializing DTO,
this macro will help you create the skeleton of DTO and assign
and/or map the values you want each field of DTO to have
from DBO/Model.

## Example

Here's how macro is used (see full sample below):
```rust
impl_dto!(
    #[derive(Debug)]    // derives
    pub struct UserResponse<UserDbo> {    // Dto<Dbo/Model>
        // dto_field: type = dbo_field OptionalMap(=> lambda with &dbo_field:type -> returns mapped_type),
        user_id: String = id => TableId::get_id,
        first_name: String = full_name => UserDbo::get_first_name,
        last_name: String = full_name => UserDbo::get_last_name,
        email_id: String = email,
        age: u8 = age => |a: &Age| a.0,
    }
);
```

The `impl_dto` macro creates the following wrappers (see full sample below):
```rust
// ...
struct UserResponse { id: String, first_name: String, last_name: String, email_id: String, age: u8 };

/// All of the below implements [`serde::Serialize`]
struct _UserResponse(pub UserDbo);
struct _UserResponseRef<'a>(pub &'a UserDbo);
struct _UserResponseOption(pub Option<UserDbo>);
struct _UserResponseRefOption<'a>(pub Option<&'a UserDbo>);
struct _UserResponseOptionRef<'a>(pub &'a Option<UserDbo>);
struct _UserResponseVec(pub Vec<UserDbo>);
struct _UserResponseRefVec<'a>(pub Vec<&'a UserDbo>);
struct _UserResponseVecRef<'a>(pub &'a Vec<UserDbo>);
// ...
```

Sample code:
```rust
mod datastore {
    #[derive(Debug)]
    pub struct TableId {
        pub table: String,
        pub id: String,
    }
    impl TableId {
        pub fn get_id(t: &TableId) -> &str {
            &t.id
        }
    }

    #[derive(Debug)]
    pub struct Age(pub u8);

    #[derive(Debug)]
    pub struct UserDbo {
        pub id: TableId,
        pub full_name: String,
        pub email: String,
        pub password: String,
        pub hash: String,
        pub age: Age,
    }
    impl UserDbo {
        pub fn get_first_name(name: &str) -> &str {
            name.split(" ").nth(0).unwrap()
        }
        pub fn get_last_name(name: &str) -> &str {
            name.split(" ").nth(1).unwrap()
        }
    }
}

mod dto {
    use super::datastore::*;
    use ser_mapper::impl_dto;

    impl_dto!(
        #[derive(Debug)]
        pub struct UserResponse<UserDbo> {
            user_id: String = id => TableId::get_id,
            first_name: String = full_name => UserDbo::get_first_name,
            last_name: String = full_name => UserDbo::get_last_name,
            email_id: String = email,
            age: u8 = age => |a: &Age| a.0,
        }
    );
}

fn main() {
    // Some DBO retrieved from DB
    let dbo = datastore::UserDbo {
        id: datastore::TableId {
            table: String::from("User"),
            id: String::from("abcd_123"),
        },
        full_name: String::from("John Doe"),
        email: String::from("jd@email.com"),
        password: String::from("hjhy7f98i4398e3328#98"),
        hash: String::from("dsjjdjjdfjdjdj"),
        age: datastore::Age(69),
    };

    // Instead of mapping, use either of the wrappers created by `impl_dto` macro
    let dto = dto::_UserResponse(dbo);

    // and it will serialize as written, for response
    assert_eq!(
        r#"{
  "user_id": "abcd_123",
  "first_name": "John",
  "last_name": "Doe",
  "email_id": "jd@email.com",
  "age": 69
}"#,
        serde_json::to_string_pretty(&dto).unwrap()
    );
}
```
