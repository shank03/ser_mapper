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

> Breaking change from `0.1.0`: Implicit lambda is replaced with lambda token expression.
>
> From below instance, `user_id: String = id => TableId::get_id,` is <br>
> replaced with `user_id: String = id => |id: &TableId| -> &str { &id.id }`.<br><br>
> Notice unlike actual lambda, this lambda token expression treats `&TableId` and `&str` as same lifetimes.<br>
> This lambda token expression is expanded into function to support same lifetimes, removing need of `TableId::get_id`.

Here's how macro is used (see full sample below):
```rust
impl_dto!(
    #[derive(Debug)]    // derives
    pub struct UserResponse<UserDbo> {    // Dto<Dbo/Model>
        // dto_field: type = dbo_field Optional(=> |var: dbo_type| -> ser_type { expr }),
        user_id: String = id => |id: &TableId| -> &str { &id.id },
        first_name: String = full_name => |n: &str| -> &str { n.split(" ").nth(0).unwrap() },
        last_name: String = full_name => |n: &str| -> &str { n.split(" ").nth(1).unwrap() },
        email_id: String = email,
        age: u8 = age => |a: &Age| -> &u8 { &a.0 },
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

    pub fn get_dbo() -> UserDbo {
        UserDbo {
            id: TableId {
                table: String::from("User"),
                id: String::from("abcd_123"),
            },
            full_name: String::from("John Doe"),
            email: String::from("jd@email.com"),
            password: String::from("password"),
            hash: String::from("hash"),
            age: Age(69),
        }
    }
}

mod dto {
    use super::datastore::*;
    use ser_mapper::impl_dto;

    impl_dto!(
        #[derive(Debug)]
        pub struct UserResponseDto<UserDbo> {
            user_id: String = id => |id: &TableId| -> &str { &id.id },
            first_name: String = full_name => |n: &str| -> &str { n.split(" ").nth(0).unwrap() },
            last_name: String = full_name => |n: &str| -> &str { n.split(" ").nth(1).unwrap() },
            email_id: String = email,
            age: u8 = age => |a: &Age| -> &u8 { &a.0 },
        }
    );
}

fn main() {
    // Some DBO retrieved from DB
    let dbo = datastore::get_dbo();

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
