pub trait Module {
    fn name(&self) -> &'static str;
    fn init(&mut self) -> anyhow::Result<()>;
}