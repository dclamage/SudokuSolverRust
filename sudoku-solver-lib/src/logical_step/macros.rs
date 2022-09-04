/// This macro is helpful for adding a logical step through the Option without computing
/// the step description if it is not needed.
#[allow(unused_macros)]
macro_rules! add_step {
    ($self:ident, $desc:ident, $desc_fn:expr) => {
        if $desc.is_some() {
            let desc_str = format!("{}: {}", $self.name(), $desc_fn);
            $desc.as_mut().unwrap().add_step(&desc_str);
        }
    };
}
#[allow(unused_imports)]
pub(crate) use add_step;

/// This macro is helpful for adding a logical step through the Option without computing
/// the step description if it is not needed.
#[allow(unused_macros)]
macro_rules! add_step_with_substeps {
    ($self:ident, $desc:ident, $desc_fn:expr, $sub_steps:expr) => {
        if $desc.is_some() {
            let desc_str = format!("{}: {}", $self.name(), $desc_fn);
            $desc
                .as_mut()
                .unwrap()
                .add_step_with_substeps(&desc_str, $sub_steps);
        }
    };
}
#[allow(unused_imports)]
pub(crate) use add_step_with_substeps;

/// This macro is helpful for adding a logical step through the Option without computing
/// the step description if it is not needed.
#[allow(unused_macros)]
macro_rules! add_step_with_elims {
    ($self:ident, $desc:ident, $desc_fn:expr, $elimination_list:expr) => {
        if $desc.is_some() {
            let desc_str = format!("{}: {}", $self.name(), $desc_fn);
            $desc
                .as_mut()
                .unwrap()
                .add_step_with_elims(&desc_str, $elimination_list);
        }
    };
}
#[allow(unused_imports)]
pub(crate) use add_step_with_elims;
