# `ser_mapper`

Serialization Mapper.

Implement mapped DTO serialzation wrapper for DBO/Model without mapping objects.

Instead of mapping DBO/Model to DTO then serializing DTO,
this macro will help you create the skeleton of DTO and assign
and/or map the values you want each field of DTO to have
from DBO/Model.

## Example

The `impl_dto` in sample code below, creates the following wrappers:
```rust
// ...
struct UserResponse { id: String, first_name: String, last_name: String, email_id: String, age: u8 };

/// All of the below implements [`serde::Serialize`]
struct _UserResponse(pub UserDbo);
struct _UserResponseRef<'a>(pub &'a UserDbo);
struct _UserResponseRefOption<'a>(pub &'a Option<UserDbo>);
struct _UserResponseOptionRef<'a>(pub Option<&'a UserDbo>);
struct _UserResponseVec(pub Vec<UserDbo>);
struct _UserResponseVecRef<'a>(pub Vec<&'a UserDbo>);
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
