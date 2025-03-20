#[macro_export]
macro_rules! define_wrapper {
    (
        $(#[$m:meta])*
        $vis:vis struct $dto:ident {
            $(
                $(#[$field_m:meta])*
                $field_vis:vis $field:ident: $field_ty:ty,
            )*
        } = $inner_entity:ty
    ) => {
        $(#[$m])*
        $vis struct $dto {
            $(
                $(#[$field_m])*
                $field_vis $field: $field_ty,
            )+
        }

        paste::paste! {
            trait [<Ser $dto>] {
                fn dto_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer;
            }

            $vis struct [<_ $dto Ref>]<'a, T = $inner_entity>(pub &'a T);
            impl<'a, T: [<Ser $dto>]> Serialize for [<_ $dto Ref>]<'a, T> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    self.0.dto_serialize(serializer)
                }
            }

            $vis struct [<_ $dto RefOption>]<'a, T = $inner_entity>(pub &'a Option<T>);
            impl<'a, T: [<Ser $dto>]> Serialize for [<_ $dto RefOption>]<'a, T> {
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

            $vis struct [<_ $dto OptionRef>]<'a, T = $inner_entity>(pub Option<&'a T>);
            impl<'a, T: [<Ser $dto>]> Serialize for [<_ $dto OptionRef>]<'a, T> {
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

            $vis struct [<_ $dto>]<T = $inner_entity>(pub T);
            impl<T: [<Ser $dto>]> Serialize for [<_ $dto>]<T> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    self.0.dto_serialize(serializer)
                }
            }

            $vis struct [<_ $dto VecRef>]<'a, T = $inner_entity>(pub &'a Vec<T>);
            impl<'a, T: [<Ser $dto>]> Serialize for [<_ $dto VecRef>]<'a, T> {
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

            $vis struct [<_ $dto Vec>]<T = $inner_entity>(pub Vec<T>);
            impl<T: [<Ser $dto>]> Serialize for [<_ $dto Vec>]<T> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    [<_ $dto VecRef>](&self.0).serialize(serializer)
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_dto {
    // $(<$($l:lifetime),*>)?
    // $(<$($generic $(= $g_ty)?),*>)?
    (
        $(#[$m:meta])*
		$vis:vis struct $dto:ident {
            $(
                $(#[$field_m:meta])*
                $field_vis:vis $field:ident: $field_ty:ty = $($inner_path:ident).+ $(=> $st_expr:expr)?,
            )*
        } $inner_entity:ty
    ) => {
        def_dto!(
            $(#[$m])*
            $vis struct $dto {
                $(
                    $(#[$field_m])*
                    $field_vis $field: $field_ty,
                )+
            } = $inner_entity
        );

        paste::paste! {
            impl [<Ser $dto>] for $inner_entity {
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

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
