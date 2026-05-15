use tyzen::Type;

#[derive(Type)]
#[tyzen(apply = "NonExistent")]
pub enum BadError {
    Variant,
}

fn main() {}
