#[derive(Debug)]
pub enum TaskSpec {
    All,
    List(Vec<String>),
}
