use tyzen::Type;

#[derive(Type)]
struct InvalidOptional {
    #[tyzen(optional)]
    count: u32,
}

fn main() {}
